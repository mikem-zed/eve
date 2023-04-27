// Copyright (c) 2022 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

// measure-config application to measure a content of /config into a PCR
// it does nothing on devices without TPM
package main

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"log"
	"math"
	"os"
	"path/filepath"
	"sort"

	tpm2log "github.com/canonical/go-tpm2"
	"github.com/canonical/tcglog-parser"
	"github.com/google/go-tpm/tpm2"
	"github.com/google/go-tpm/tpmutil"
)

const (
	//TpmDevicePath is the TPM device file path
	TpmDevicePath   = "/dev/tpmrm0"
	rootfsPCRIndex  = 13
	rootfsPCRHandle = tpmutil.Handle(tpm2.PCRFirst + rootfsPCRIndex)
	configPCRIndex  = 14
	configPCRHandle = tpmutil.Handle(tpm2.PCRFirst + configPCRIndex)
	//PCREvent (TPM2_PCR_Event) supports event size of maximum 1024 bytes.
	maxEventDataSize = 1024
)

type fileInfo struct {
	exist          bool
	measureContent bool
}

type tpmEvent struct {
	rawEventData
	data   string
	pcrMap map[tpm2log.HashAlgorithmId][]byte
	pcr    tcglog.PCRIndex
}

type rawEventData []byte

func (b rawEventData) Bytes() []byte {
	return []byte(b)
}

// implementing the EventData interface
func (e *tpmEvent) String() string {
	return fmt.Sprintf("%s", e.data)
}

func (e *tpmEvent) Write(w io.Writer) error {
	_, err := io.WriteString(w, fmt.Sprintf("%s\x00", e.data))
	return err
}

func (e *tpmEvent) ToTCGEvent() *tcglog.Event {
	digests := make(tcglog.DigestMap)
	for alg, digest := range e.pcrMap {
		digests[alg] = digest
	}
	event := tcglog.Event{
		PCRIndex:  e.pcr,
		EventType: tcglog.EventTypeIPL,
		Digests:   digests,
		Data:      e,
	}
	return &event
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

// we do not measure content of following files
// because they are unique for each device
func getExcludeList() []string {
	return []string{
		"/config/tpm_credential",
		"/config/device.cert.pem",
		"/config/device.key.pem",
		"/config/onboard.cert.pem",
		"/config/onboard.key.pem",
		"/config/soft_serial",
	}
}

func isInExcludeList(path string) bool {
	for _, file := range getExcludeList() {
		if file == path {
			return true
		}
	}
	return false
}

// these file may appear later on the device and we record the
// fact that file exists. during attestation process we can detect
// this fact by comparing saved and current event log
func getDangerousList() []string {
	return []string{
		"/config/bootstrap-config.pb",
		"/config/DevicePortConfig/override.json",
		"/config/GlobalConfig/global.json",
		"/config/Force-API-V1",
	}
}

func sha256sumForFile(filePath string) (string, error) {
	file, err := os.Open(filePath)
	if err != nil {
		return "", err
	}
	defer file.Close()

	hash := sha256.New()
	if _, err := io.Copy(hash, file); err != nil {
		return "", err
	}
	return hex.EncodeToString(hash.Sum(nil)), nil
}

func performMeasurement(filePath string, tpm io.ReadWriter, exist bool, content bool, algos []tpm2log.HashAlgorithmId) (*tpmEvent, error) {
	var eventData string
	if content {
		hash, err := sha256sumForFile(filePath)
		if err != nil {
			return nil, fmt.Errorf("cannot measure %s :%v", filePath, err)
		}
		eventData = fmt.Sprintf("file:%s exist:true content-hash:%s", filePath, hash)
	} else {
		eventData = fmt.Sprintf("file:%s exist:%t", filePath, exist)
	}

	// Loop over the data and if it is larger than 1024 (max size PCREvent consumes)
	// break it into 1024 bytes chunks, otherwise just loop once and pass data to PCREvent.
	for offset, length := 0, 0; offset < len(eventData); offset += length {
		length = min(maxEventDataSize, len(eventData)-offset)
		// PCREvent internally hashes the data with all supported algorithms
		// associated with the PCR banks, and extends them all before return.
		err := tpm2.PCREvent(tpm, configPCRHandle, []byte(eventData[offset:offset+length]))
		if err != nil {
			return nil, fmt.Errorf("cannot measure %s. couldn't extend PCR: %v", filePath, err)
		}
	}

	pcr, err := readEvePCR(tpm, configPCRIndex, algos)
	if err != nil {
		return nil, fmt.Errorf("cannot measure %s. couldn't read PCR: %v", filePath, err)
	}

	return &tpmEvent{data: eventData, pcrMap: pcr}, nil
}

func getFileMap() (map[string]fileInfo, error) {
	files := make(map[string]fileInfo)

	walkErr := filepath.Walk("/config",
		func(path string, info os.FileInfo, err error) error {
			if err != nil {
				return err
			}

			if info.IsDir() {
				return nil
			}
			// may mark file as excluded but we will measure presence/absence
			files[path] = fileInfo{exist: true, measureContent: !isInExcludeList(path)}
			return nil
		})
	if walkErr != nil {
		return nil, walkErr
	}

	// for every file in both exclude and risky lists add entries so the list of files
	// is always the same across all devices in the world
	for _, file := range getExcludeList() {
		_, found := files[file]
		if !found {
			files[file] = fileInfo{exist: false, measureContent: false}
		}
	}

	for _, file := range getDangerousList() {
		_, found := files[file]
		if !found {
			files[file] = fileInfo{exist: false, measureContent: false}
		}
	}

	return files, nil
}

func getSortedFileList(files map[string]fileInfo) []string {
	keys := make([]string, 0, len(files))
	for k := range files {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return keys
}

// allocatedPCRBanks returns a list of selections corresponding to the TPM's implemented PCRs.
func allocatedPCRBanks(rw io.ReadWriter) ([]tpm2.PCRSelection, error) {
	caps, moreData, err := tpm2.GetCapability(rw, tpm2.CapabilityPCRs, math.MaxUint32, 0)
	if err != nil {
		return nil, fmt.Errorf("listing implemented PCR banks: %w", err)
	}
	if moreData {
		return nil, fmt.Errorf("extra data from GetCapability")
	}
	var sels []tpm2.PCRSelection
	for _, cap := range caps {
		sel, ok := cap.(tpm2.PCRSelection)
		if !ok {
			return nil, fmt.Errorf("unexpected data from GetCapability")
		}
		// skip empty (unallocated) PCR selections
		if len(sel.PCRs) == 0 {
			continue
		}
		sels = append(sels, sel)
	}
	return sels, nil
}

func createTpmEventLog(tpm io.ReadWriter) (*tcglog.Log, []tpm2log.HashAlgorithmId, error) {

	var algos []tpm2log.HashAlgorithmId
	var digestSizes []tcglog.EFISpecIdEventAlgorithmSize
	pcrSelection, err := allocatedPCRBanks(tpm)

	if err != nil {
		return nil, nil, fmt.Errorf("cannot create TPM event log (couldn't read enabled PCRs): %v", err)
	}

	for _, alg := range pcrSelection {
		algId := tpm2log.HashAlgorithmId(alg.Hash)
		algos = append(algos, algId)
		digestSizes = append(digestSizes, tcglog.EFISpecIdEventAlgorithmSize{
			AlgorithmId: algId,
			DigestSize:  uint16(algId.Size()),
		})
	}

	event := tcglog.Event{
		PCRIndex:  0,
		EventType: tcglog.EventTypeNoAction,
		// this event always has only SHA1 digest for backwards compatibility with TPM 1.2
		Digests: tcglog.DigestMap{tpm2log.HashAlgorithmSHA1: make(tcglog.Digest, tpm2log.HashAlgorithmSHA1.Size())},
		Data: &tcglog.SpecIdEvent03{
			SpecVersionMajor: 2,
			UintnSize:        2, //FIXME: is this the size of the following array?
			DigestSizes:      digestSizes}}

	tcg_log, _ := tcglog.NewLog(&event)
	return tcg_log, algos, nil
}

func measureConfig(tpm io.ReadWriter) error {
	files, err := getFileMap()

	if err != nil {
		return fmt.Errorf("cannot get file list: %v", err)
	}

	eventLog, digests, err := createTpmEventLog(tpm)

	if err != nil {
		return fmt.Errorf("cannot create TPM event log: %v", err)
	}

	//TODO: read root fs measurements and create TPM event
	//readEvePCR(tpm, rootfsPCRIndex, digests)

	//get sorted list of files. We must always go the same order
	//otherwise we'll get different PCR value even with exactly the same
	//file names and their content
	fileNames := getSortedFileList(files)

	for _, file := range fileNames {
		info := files[file]
		var event *tpmEvent

		if info.exist {
			if info.measureContent {
				event, err = performMeasurement(file, tpm, true, true, digests)
			} else {
				event, err = performMeasurement(file, tpm, true, false, digests)
			}
		} else {
			event, err = performMeasurement(file, tpm, false, false, digests)
		}
		if err != nil {
			return fmt.Errorf("cannot measure %s: %v", file, err)
		}
		//Now we have a new value of PCR and an event
		//TODO: add events to the event log, if event data exceeds 1024 bytes,
		// make sure to break it into 1024 bytes chunks with added indicators
		// (e.g. part n of m) to be able to reconstruct the even data for validation.
		// for now we just print our measurements to boot log.
		for i, alg := range event.pcrMap {
			log.Printf("%s pcr: %d %s", event.data, i, hex.EncodeToString(alg))
		}
		eventLog.Events = append(eventLog.Events, event.ToTCGEvent())
	}

	fOut, err := os.Create("/run/measurefs/tpm_log.bin")
	if err != nil {
		return fmt.Errorf("cannot create EVE TPM log file: %v", err)
	}
	defer fOut.Close()

	err = eventLog.Write(fOut)

	if err != nil {
		return fmt.Errorf("cannot write log to EVE TPM log file: %v", err)
	}

	return nil
}

func readEvePCR(tpm io.ReadWriter, pcr int, algos []tpm2log.HashAlgorithmId) (map[tpm2log.HashAlgorithmId][]byte, error) {

	pcrMap := make(map[tpm2log.HashAlgorithmId][]byte)

	for _, alg := range algos {
		algId, err := tpm2.HashToAlgorithm(alg.GetHash())
		if err != nil {
			return nil, fmt.Errorf("cannot convert hash to algorithm: %v", err)
		}
		pcrVal, err := tpm2.ReadPCR(tpm, pcr, algId)
		if err != nil {
			return nil, fmt.Errorf("cannot read PCR %d, alg %d : %v", pcr, alg, err)
		}
		pcrMap[alg] = pcrVal
	}

	return pcrMap, nil
}

// Some file like generated certificates do not exist during the installation.
// do we care? it seems nobody is using eve just after installation.
// live image won't report the same PCR values as installed EVE
func main() {
	tpm, err := tpm2.OpenTPM(TpmDevicePath)
	if err != nil {
		log.Printf("couldn't open TPM device %s. Exiting", TpmDevicePath)
		return
	}
	defer tpm.Close()

	err = measureConfig(tpm)

	if err != nil {
		log.Fatal(err)
	}
}
