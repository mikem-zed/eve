// Copyright(c) 2020 Zededa, Inc.
// All rights reserved.

// Code generated by protoc-gen-go. DO NOT EDIT.
// versions:
// 	protoc-gen-go v1.28.1
// 	protoc        v4.23.4
// source: certs/certs.proto

package certs

import (
	evecommon "github.com/lf-edge/eve-api/go/evecommon"
	protoreflect "google.golang.org/protobuf/reflect/protoreflect"
	protoimpl "google.golang.org/protobuf/runtime/protoimpl"
	reflect "reflect"
	sync "sync"
)

const (
	// Verify that this generated code is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(20 - protoimpl.MinVersion)
	// Verify that runtime/protoimpl is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(protoimpl.MaxVersion - 20)
)

type ZCertMetaDataType int32

const (
	ZCertMetaDataType_Z_CERT_META_DATA_TYPE_INVALID     ZCertMetaDataType = 0
	ZCertMetaDataType_Z_CERT_META_DATA_TYPE_TPM2_PUBLIC ZCertMetaDataType = 1 //TPM2_PUBLIC blob from TPM2.0
)

// Enum value maps for ZCertMetaDataType.
var (
	ZCertMetaDataType_name = map[int32]string{
		0: "Z_CERT_META_DATA_TYPE_INVALID",
		1: "Z_CERT_META_DATA_TYPE_TPM2_PUBLIC",
	}
	ZCertMetaDataType_value = map[string]int32{
		"Z_CERT_META_DATA_TYPE_INVALID":     0,
		"Z_CERT_META_DATA_TYPE_TPM2_PUBLIC": 1,
	}
)

func (x ZCertMetaDataType) Enum() *ZCertMetaDataType {
	p := new(ZCertMetaDataType)
	*p = x
	return p
}

func (x ZCertMetaDataType) String() string {
	return protoimpl.X.EnumStringOf(x.Descriptor(), protoreflect.EnumNumber(x))
}

func (ZCertMetaDataType) Descriptor() protoreflect.EnumDescriptor {
	return file_certs_certs_proto_enumTypes[0].Descriptor()
}

func (ZCertMetaDataType) Type() protoreflect.EnumType {
	return &file_certs_certs_proto_enumTypes[0]
}

func (x ZCertMetaDataType) Number() protoreflect.EnumNumber {
	return protoreflect.EnumNumber(x)
}

// Deprecated: Use ZCertMetaDataType.Descriptor instead.
func (ZCertMetaDataType) EnumDescriptor() ([]byte, []int) {
	return file_certs_certs_proto_rawDescGZIP(), []int{0}
}

type ZCertType int32

const (
	ZCertType_CERT_TYPE_CONTROLLER_NONE ZCertType = 0
	// controller generated certificates
	ZCertType_CERT_TYPE_CONTROLLER_SIGNING       ZCertType = 1 //set for the leaf certificate used by controller to sign payload envelopes
	ZCertType_CERT_TYPE_CONTROLLER_INTERMEDIATE  ZCertType = 2 //set for intermediate certs used to validate the certificates
	ZCertType_CERT_TYPE_CONTROLLER_ECDH_EXCHANGE ZCertType = 3 //set for certificate used by controller to share any symmetric key using ECDH
	// device generated certificates
	ZCertType_CERT_TYPE_DEVICE_ONBOARDING         ZCertType = 10 //for identifying the device
	ZCertType_CERT_TYPE_DEVICE_RESTRICTED_SIGNING ZCertType = 11 //node for attestation
	ZCertType_CERT_TYPE_DEVICE_ENDORSEMENT_RSA    ZCertType = 12 //endorsement key certificate with RSASSA signing algorithm
	ZCertType_CERT_TYPE_DEVICE_ECDH_EXCHANGE      ZCertType = 13 //to share symmetric key using ECDH
)

// Enum value maps for ZCertType.
var (
	ZCertType_name = map[int32]string{
		0:  "CERT_TYPE_CONTROLLER_NONE",
		1:  "CERT_TYPE_CONTROLLER_SIGNING",
		2:  "CERT_TYPE_CONTROLLER_INTERMEDIATE",
		3:  "CERT_TYPE_CONTROLLER_ECDH_EXCHANGE",
		10: "CERT_TYPE_DEVICE_ONBOARDING",
		11: "CERT_TYPE_DEVICE_RESTRICTED_SIGNING",
		12: "CERT_TYPE_DEVICE_ENDORSEMENT_RSA",
		13: "CERT_TYPE_DEVICE_ECDH_EXCHANGE",
	}
	ZCertType_value = map[string]int32{
		"CERT_TYPE_CONTROLLER_NONE":           0,
		"CERT_TYPE_CONTROLLER_SIGNING":        1,
		"CERT_TYPE_CONTROLLER_INTERMEDIATE":   2,
		"CERT_TYPE_CONTROLLER_ECDH_EXCHANGE":  3,
		"CERT_TYPE_DEVICE_ONBOARDING":         10,
		"CERT_TYPE_DEVICE_RESTRICTED_SIGNING": 11,
		"CERT_TYPE_DEVICE_ENDORSEMENT_RSA":    12,
		"CERT_TYPE_DEVICE_ECDH_EXCHANGE":      13,
	}
)

func (x ZCertType) Enum() *ZCertType {
	p := new(ZCertType)
	*p = x
	return p
}

func (x ZCertType) String() string {
	return protoimpl.X.EnumStringOf(x.Descriptor(), protoreflect.EnumNumber(x))
}

func (ZCertType) Descriptor() protoreflect.EnumDescriptor {
	return file_certs_certs_proto_enumTypes[1].Descriptor()
}

func (ZCertType) Type() protoreflect.EnumType {
	return &file_certs_certs_proto_enumTypes[1]
}

func (x ZCertType) Number() protoreflect.EnumNumber {
	return protoreflect.EnumNumber(x)
}

// Deprecated: Use ZCertType.Descriptor instead.
func (ZCertType) EnumDescriptor() ([]byte, []int) {
	return file_certs_certs_proto_rawDescGZIP(), []int{1}
}

//  This is the response payload for GET /api/v1/edgeDevice/certs
// or /api/v2/edgeDevice/certs
// ZControllerCert carries a set of X.509 certificate and their properties
// from Controller to EVE.
type ZControllerCert struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	Certs []*ZCert `protobuf:"bytes,1,rep,name=certs,proto3" json:"certs,omitempty"` //list of certificates sent by controller
}

func (x *ZControllerCert) Reset() {
	*x = ZControllerCert{}
	if protoimpl.UnsafeEnabled {
		mi := &file_certs_certs_proto_msgTypes[0]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *ZControllerCert) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*ZControllerCert) ProtoMessage() {}

func (x *ZControllerCert) ProtoReflect() protoreflect.Message {
	mi := &file_certs_certs_proto_msgTypes[0]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use ZControllerCert.ProtoReflect.Descriptor instead.
func (*ZControllerCert) Descriptor() ([]byte, []int) {
	return file_certs_certs_proto_rawDescGZIP(), []int{0}
}

func (x *ZControllerCert) GetCerts() []*ZCert {
	if x != nil {
		return x.Certs
	}
	return nil
}

type ZCertMetaData struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	Type     ZCertMetaDataType `protobuf:"varint,1,opt,name=type,proto3,enum=org.lfedge.eve.certs.ZCertMetaDataType" json:"type,omitempty"` //meta-data type
	MetaData []byte            `protobuf:"bytes,2,opt,name=meta_data,json=metaData,proto3" json:"meta_data,omitempty"`                      //blob for the meta data
}

func (x *ZCertMetaData) Reset() {
	*x = ZCertMetaData{}
	if protoimpl.UnsafeEnabled {
		mi := &file_certs_certs_proto_msgTypes[1]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *ZCertMetaData) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*ZCertMetaData) ProtoMessage() {}

func (x *ZCertMetaData) ProtoReflect() protoreflect.Message {
	mi := &file_certs_certs_proto_msgTypes[1]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use ZCertMetaData.ProtoReflect.Descriptor instead.
func (*ZCertMetaData) Descriptor() ([]byte, []int) {
	return file_certs_certs_proto_rawDescGZIP(), []int{1}
}

func (x *ZCertMetaData) GetType() ZCertMetaDataType {
	if x != nil {
		return x.Type
	}
	return ZCertMetaDataType_Z_CERT_META_DATA_TYPE_INVALID
}

func (x *ZCertMetaData) GetMetaData() []byte {
	if x != nil {
		return x.MetaData
	}
	return nil
}

// ZCert is used for both controller certificates and edge-node certificates
type ZCert struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	HashAlgo      evecommon.HashAlgorithm `protobuf:"varint,1,opt,name=hashAlgo,proto3,enum=org.lfedge.eve.common.HashAlgorithm" json:"hashAlgo,omitempty"` //hash method used to arrive at certHash
	CertHash      []byte                  `protobuf:"bytes,2,opt,name=certHash,proto3" json:"certHash,omitempty"`                                           //truncated hash of the cert, according to hashing scheme in hashAlgo
	Type          ZCertType               `protobuf:"varint,3,opt,name=type,proto3,enum=org.lfedge.eve.certs.ZCertType" json:"type,omitempty"`              //what kind of certificate(to identify the target use case)
	Cert          []byte                  `protobuf:"bytes,4,opt,name=cert,proto3" json:"cert,omitempty"`                                                   //X509 cert in .PEM format
	Attributes    *ZCertAttr              `protobuf:"bytes,5,opt,name=attributes,proto3" json:"attributes,omitempty"`                                       //properties of this certificate
	MetaDataItems []*ZCertMetaData        `protobuf:"bytes,6,rep,name=meta_data_items,json=metaDataItems,proto3" json:"meta_data_items,omitempty"`          //Any meta-data associated with this certificate
}

func (x *ZCert) Reset() {
	*x = ZCert{}
	if protoimpl.UnsafeEnabled {
		mi := &file_certs_certs_proto_msgTypes[2]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *ZCert) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*ZCert) ProtoMessage() {}

func (x *ZCert) ProtoReflect() protoreflect.Message {
	mi := &file_certs_certs_proto_msgTypes[2]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use ZCert.ProtoReflect.Descriptor instead.
func (*ZCert) Descriptor() ([]byte, []int) {
	return file_certs_certs_proto_rawDescGZIP(), []int{2}
}

func (x *ZCert) GetHashAlgo() evecommon.HashAlgorithm {
	if x != nil {
		return x.HashAlgo
	}
	return evecommon.HashAlgorithm(0)
}

func (x *ZCert) GetCertHash() []byte {
	if x != nil {
		return x.CertHash
	}
	return nil
}

func (x *ZCert) GetType() ZCertType {
	if x != nil {
		return x.Type
	}
	return ZCertType_CERT_TYPE_CONTROLLER_NONE
}

func (x *ZCert) GetCert() []byte {
	if x != nil {
		return x.Cert
	}
	return nil
}

func (x *ZCert) GetAttributes() *ZCertAttr {
	if x != nil {
		return x.Attributes
	}
	return nil
}

func (x *ZCert) GetMetaDataItems() []*ZCertMetaData {
	if x != nil {
		return x.MetaDataItems
	}
	return nil
}

type ZCertAttr struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	IsMutable bool `protobuf:"varint,1,opt,name=is_mutable,json=isMutable,proto3" json:"is_mutable,omitempty"` //set to false for immutable certificates
	IsTpm     bool `protobuf:"varint,2,opt,name=is_tpm,json=isTpm,proto3" json:"is_tpm,omitempty"`             //generated by a TPM
}

func (x *ZCertAttr) Reset() {
	*x = ZCertAttr{}
	if protoimpl.UnsafeEnabled {
		mi := &file_certs_certs_proto_msgTypes[3]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *ZCertAttr) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*ZCertAttr) ProtoMessage() {}

func (x *ZCertAttr) ProtoReflect() protoreflect.Message {
	mi := &file_certs_certs_proto_msgTypes[3]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use ZCertAttr.ProtoReflect.Descriptor instead.
func (*ZCertAttr) Descriptor() ([]byte, []int) {
	return file_certs_certs_proto_rawDescGZIP(), []int{3}
}

func (x *ZCertAttr) GetIsMutable() bool {
	if x != nil {
		return x.IsMutable
	}
	return false
}

func (x *ZCertAttr) GetIsTpm() bool {
	if x != nil {
		return x.IsTpm
	}
	return false
}

var File_certs_certs_proto protoreflect.FileDescriptor

var file_certs_certs_proto_rawDesc = []byte{
	0x0a, 0x11, 0x63, 0x65, 0x72, 0x74, 0x73, 0x2f, 0x63, 0x65, 0x72, 0x74, 0x73, 0x2e, 0x70, 0x72,
	0x6f, 0x74, 0x6f, 0x12, 0x14, 0x6f, 0x72, 0x67, 0x2e, 0x6c, 0x66, 0x65, 0x64, 0x67, 0x65, 0x2e,
	0x65, 0x76, 0x65, 0x2e, 0x63, 0x65, 0x72, 0x74, 0x73, 0x1a, 0x19, 0x65, 0x76, 0x65, 0x63, 0x6f,
	0x6d, 0x6d, 0x6f, 0x6e, 0x2f, 0x65, 0x76, 0x65, 0x63, 0x6f, 0x6d, 0x6d, 0x6f, 0x6e, 0x2e, 0x70,
	0x72, 0x6f, 0x74, 0x6f, 0x22, 0x44, 0x0a, 0x0f, 0x5a, 0x43, 0x6f, 0x6e, 0x74, 0x72, 0x6f, 0x6c,
	0x6c, 0x65, 0x72, 0x43, 0x65, 0x72, 0x74, 0x12, 0x31, 0x0a, 0x05, 0x63, 0x65, 0x72, 0x74, 0x73,
	0x18, 0x01, 0x20, 0x03, 0x28, 0x0b, 0x32, 0x1b, 0x2e, 0x6f, 0x72, 0x67, 0x2e, 0x6c, 0x66, 0x65,
	0x64, 0x67, 0x65, 0x2e, 0x65, 0x76, 0x65, 0x2e, 0x63, 0x65, 0x72, 0x74, 0x73, 0x2e, 0x5a, 0x43,
	0x65, 0x72, 0x74, 0x52, 0x05, 0x63, 0x65, 0x72, 0x74, 0x73, 0x22, 0x69, 0x0a, 0x0d, 0x5a, 0x43,
	0x65, 0x72, 0x74, 0x4d, 0x65, 0x74, 0x61, 0x44, 0x61, 0x74, 0x61, 0x12, 0x3b, 0x0a, 0x04, 0x74,
	0x79, 0x70, 0x65, 0x18, 0x01, 0x20, 0x01, 0x28, 0x0e, 0x32, 0x27, 0x2e, 0x6f, 0x72, 0x67, 0x2e,
	0x6c, 0x66, 0x65, 0x64, 0x67, 0x65, 0x2e, 0x65, 0x76, 0x65, 0x2e, 0x63, 0x65, 0x72, 0x74, 0x73,
	0x2e, 0x5a, 0x43, 0x65, 0x72, 0x74, 0x4d, 0x65, 0x74, 0x61, 0x44, 0x61, 0x74, 0x61, 0x54, 0x79,
	0x70, 0x65, 0x52, 0x04, 0x74, 0x79, 0x70, 0x65, 0x12, 0x1b, 0x0a, 0x09, 0x6d, 0x65, 0x74, 0x61,
	0x5f, 0x64, 0x61, 0x74, 0x61, 0x18, 0x02, 0x20, 0x01, 0x28, 0x0c, 0x52, 0x08, 0x6d, 0x65, 0x74,
	0x61, 0x44, 0x61, 0x74, 0x61, 0x22, 0xbc, 0x02, 0x0a, 0x05, 0x5a, 0x43, 0x65, 0x72, 0x74, 0x12,
	0x40, 0x0a, 0x08, 0x68, 0x61, 0x73, 0x68, 0x41, 0x6c, 0x67, 0x6f, 0x18, 0x01, 0x20, 0x01, 0x28,
	0x0e, 0x32, 0x24, 0x2e, 0x6f, 0x72, 0x67, 0x2e, 0x6c, 0x66, 0x65, 0x64, 0x67, 0x65, 0x2e, 0x65,
	0x76, 0x65, 0x2e, 0x63, 0x6f, 0x6d, 0x6d, 0x6f, 0x6e, 0x2e, 0x48, 0x61, 0x73, 0x68, 0x41, 0x6c,
	0x67, 0x6f, 0x72, 0x69, 0x74, 0x68, 0x6d, 0x52, 0x08, 0x68, 0x61, 0x73, 0x68, 0x41, 0x6c, 0x67,
	0x6f, 0x12, 0x1a, 0x0a, 0x08, 0x63, 0x65, 0x72, 0x74, 0x48, 0x61, 0x73, 0x68, 0x18, 0x02, 0x20,
	0x01, 0x28, 0x0c, 0x52, 0x08, 0x63, 0x65, 0x72, 0x74, 0x48, 0x61, 0x73, 0x68, 0x12, 0x33, 0x0a,
	0x04, 0x74, 0x79, 0x70, 0x65, 0x18, 0x03, 0x20, 0x01, 0x28, 0x0e, 0x32, 0x1f, 0x2e, 0x6f, 0x72,
	0x67, 0x2e, 0x6c, 0x66, 0x65, 0x64, 0x67, 0x65, 0x2e, 0x65, 0x76, 0x65, 0x2e, 0x63, 0x65, 0x72,
	0x74, 0x73, 0x2e, 0x5a, 0x43, 0x65, 0x72, 0x74, 0x54, 0x79, 0x70, 0x65, 0x52, 0x04, 0x74, 0x79,
	0x70, 0x65, 0x12, 0x12, 0x0a, 0x04, 0x63, 0x65, 0x72, 0x74, 0x18, 0x04, 0x20, 0x01, 0x28, 0x0c,
	0x52, 0x04, 0x63, 0x65, 0x72, 0x74, 0x12, 0x3f, 0x0a, 0x0a, 0x61, 0x74, 0x74, 0x72, 0x69, 0x62,
	0x75, 0x74, 0x65, 0x73, 0x18, 0x05, 0x20, 0x01, 0x28, 0x0b, 0x32, 0x1f, 0x2e, 0x6f, 0x72, 0x67,
	0x2e, 0x6c, 0x66, 0x65, 0x64, 0x67, 0x65, 0x2e, 0x65, 0x76, 0x65, 0x2e, 0x63, 0x65, 0x72, 0x74,
	0x73, 0x2e, 0x5a, 0x43, 0x65, 0x72, 0x74, 0x41, 0x74, 0x74, 0x72, 0x52, 0x0a, 0x61, 0x74, 0x74,
	0x72, 0x69, 0x62, 0x75, 0x74, 0x65, 0x73, 0x12, 0x4b, 0x0a, 0x0f, 0x6d, 0x65, 0x74, 0x61, 0x5f,
	0x64, 0x61, 0x74, 0x61, 0x5f, 0x69, 0x74, 0x65, 0x6d, 0x73, 0x18, 0x06, 0x20, 0x03, 0x28, 0x0b,
	0x32, 0x23, 0x2e, 0x6f, 0x72, 0x67, 0x2e, 0x6c, 0x66, 0x65, 0x64, 0x67, 0x65, 0x2e, 0x65, 0x76,
	0x65, 0x2e, 0x63, 0x65, 0x72, 0x74, 0x73, 0x2e, 0x5a, 0x43, 0x65, 0x72, 0x74, 0x4d, 0x65, 0x74,
	0x61, 0x44, 0x61, 0x74, 0x61, 0x52, 0x0d, 0x6d, 0x65, 0x74, 0x61, 0x44, 0x61, 0x74, 0x61, 0x49,
	0x74, 0x65, 0x6d, 0x73, 0x22, 0x41, 0x0a, 0x09, 0x5a, 0x43, 0x65, 0x72, 0x74, 0x41, 0x74, 0x74,
	0x72, 0x12, 0x1d, 0x0a, 0x0a, 0x69, 0x73, 0x5f, 0x6d, 0x75, 0x74, 0x61, 0x62, 0x6c, 0x65, 0x18,
	0x01, 0x20, 0x01, 0x28, 0x08, 0x52, 0x09, 0x69, 0x73, 0x4d, 0x75, 0x74, 0x61, 0x62, 0x6c, 0x65,
	0x12, 0x15, 0x0a, 0x06, 0x69, 0x73, 0x5f, 0x74, 0x70, 0x6d, 0x18, 0x02, 0x20, 0x01, 0x28, 0x08,
	0x52, 0x05, 0x69, 0x73, 0x54, 0x70, 0x6d, 0x2a, 0x5d, 0x0a, 0x11, 0x5a, 0x43, 0x65, 0x72, 0x74,
	0x4d, 0x65, 0x74, 0x61, 0x44, 0x61, 0x74, 0x61, 0x54, 0x79, 0x70, 0x65, 0x12, 0x21, 0x0a, 0x1d,
	0x5a, 0x5f, 0x43, 0x45, 0x52, 0x54, 0x5f, 0x4d, 0x45, 0x54, 0x41, 0x5f, 0x44, 0x41, 0x54, 0x41,
	0x5f, 0x54, 0x59, 0x50, 0x45, 0x5f, 0x49, 0x4e, 0x56, 0x41, 0x4c, 0x49, 0x44, 0x10, 0x00, 0x12,
	0x25, 0x0a, 0x21, 0x5a, 0x5f, 0x43, 0x45, 0x52, 0x54, 0x5f, 0x4d, 0x45, 0x54, 0x41, 0x5f, 0x44,
	0x41, 0x54, 0x41, 0x5f, 0x54, 0x59, 0x50, 0x45, 0x5f, 0x54, 0x50, 0x4d, 0x32, 0x5f, 0x50, 0x55,
	0x42, 0x4c, 0x49, 0x43, 0x10, 0x01, 0x2a, 0xaf, 0x02, 0x0a, 0x09, 0x5a, 0x43, 0x65, 0x72, 0x74,
	0x54, 0x79, 0x70, 0x65, 0x12, 0x1d, 0x0a, 0x19, 0x43, 0x45, 0x52, 0x54, 0x5f, 0x54, 0x59, 0x50,
	0x45, 0x5f, 0x43, 0x4f, 0x4e, 0x54, 0x52, 0x4f, 0x4c, 0x4c, 0x45, 0x52, 0x5f, 0x4e, 0x4f, 0x4e,
	0x45, 0x10, 0x00, 0x12, 0x20, 0x0a, 0x1c, 0x43, 0x45, 0x52, 0x54, 0x5f, 0x54, 0x59, 0x50, 0x45,
	0x5f, 0x43, 0x4f, 0x4e, 0x54, 0x52, 0x4f, 0x4c, 0x4c, 0x45, 0x52, 0x5f, 0x53, 0x49, 0x47, 0x4e,
	0x49, 0x4e, 0x47, 0x10, 0x01, 0x12, 0x25, 0x0a, 0x21, 0x43, 0x45, 0x52, 0x54, 0x5f, 0x54, 0x59,
	0x50, 0x45, 0x5f, 0x43, 0x4f, 0x4e, 0x54, 0x52, 0x4f, 0x4c, 0x4c, 0x45, 0x52, 0x5f, 0x49, 0x4e,
	0x54, 0x45, 0x52, 0x4d, 0x45, 0x44, 0x49, 0x41, 0x54, 0x45, 0x10, 0x02, 0x12, 0x26, 0x0a, 0x22,
	0x43, 0x45, 0x52, 0x54, 0x5f, 0x54, 0x59, 0x50, 0x45, 0x5f, 0x43, 0x4f, 0x4e, 0x54, 0x52, 0x4f,
	0x4c, 0x4c, 0x45, 0x52, 0x5f, 0x45, 0x43, 0x44, 0x48, 0x5f, 0x45, 0x58, 0x43, 0x48, 0x41, 0x4e,
	0x47, 0x45, 0x10, 0x03, 0x12, 0x1f, 0x0a, 0x1b, 0x43, 0x45, 0x52, 0x54, 0x5f, 0x54, 0x59, 0x50,
	0x45, 0x5f, 0x44, 0x45, 0x56, 0x49, 0x43, 0x45, 0x5f, 0x4f, 0x4e, 0x42, 0x4f, 0x41, 0x52, 0x44,
	0x49, 0x4e, 0x47, 0x10, 0x0a, 0x12, 0x27, 0x0a, 0x23, 0x43, 0x45, 0x52, 0x54, 0x5f, 0x54, 0x59,
	0x50, 0x45, 0x5f, 0x44, 0x45, 0x56, 0x49, 0x43, 0x45, 0x5f, 0x52, 0x45, 0x53, 0x54, 0x52, 0x49,
	0x43, 0x54, 0x45, 0x44, 0x5f, 0x53, 0x49, 0x47, 0x4e, 0x49, 0x4e, 0x47, 0x10, 0x0b, 0x12, 0x24,
	0x0a, 0x20, 0x43, 0x45, 0x52, 0x54, 0x5f, 0x54, 0x59, 0x50, 0x45, 0x5f, 0x44, 0x45, 0x56, 0x49,
	0x43, 0x45, 0x5f, 0x45, 0x4e, 0x44, 0x4f, 0x52, 0x53, 0x45, 0x4d, 0x45, 0x4e, 0x54, 0x5f, 0x52,
	0x53, 0x41, 0x10, 0x0c, 0x12, 0x22, 0x0a, 0x1e, 0x43, 0x45, 0x52, 0x54, 0x5f, 0x54, 0x59, 0x50,
	0x45, 0x5f, 0x44, 0x45, 0x56, 0x49, 0x43, 0x45, 0x5f, 0x45, 0x43, 0x44, 0x48, 0x5f, 0x45, 0x58,
	0x43, 0x48, 0x41, 0x4e, 0x47, 0x45, 0x10, 0x0d, 0x42, 0x3b, 0x0a, 0x14, 0x6f, 0x72, 0x67, 0x2e,
	0x6c, 0x66, 0x65, 0x64, 0x67, 0x65, 0x2e, 0x65, 0x76, 0x65, 0x2e, 0x63, 0x65, 0x72, 0x74, 0x73,
	0x5a, 0x23, 0x67, 0x69, 0x74, 0x68, 0x75, 0x62, 0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x6c, 0x66, 0x2d,
	0x65, 0x64, 0x67, 0x65, 0x2f, 0x65, 0x76, 0x65, 0x2d, 0x61, 0x70, 0x69, 0x2f, 0x67, 0x6f, 0x2f,
	0x63, 0x65, 0x72, 0x74, 0x73, 0x62, 0x06, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x33,
}

var (
	file_certs_certs_proto_rawDescOnce sync.Once
	file_certs_certs_proto_rawDescData = file_certs_certs_proto_rawDesc
)

func file_certs_certs_proto_rawDescGZIP() []byte {
	file_certs_certs_proto_rawDescOnce.Do(func() {
		file_certs_certs_proto_rawDescData = protoimpl.X.CompressGZIP(file_certs_certs_proto_rawDescData)
	})
	return file_certs_certs_proto_rawDescData
}

var file_certs_certs_proto_enumTypes = make([]protoimpl.EnumInfo, 2)
var file_certs_certs_proto_msgTypes = make([]protoimpl.MessageInfo, 4)
var file_certs_certs_proto_goTypes = []interface{}{
	(ZCertMetaDataType)(0),       // 0: org.lfedge.eve.certs.ZCertMetaDataType
	(ZCertType)(0),               // 1: org.lfedge.eve.certs.ZCertType
	(*ZControllerCert)(nil),      // 2: org.lfedge.eve.certs.ZControllerCert
	(*ZCertMetaData)(nil),        // 3: org.lfedge.eve.certs.ZCertMetaData
	(*ZCert)(nil),                // 4: org.lfedge.eve.certs.ZCert
	(*ZCertAttr)(nil),            // 5: org.lfedge.eve.certs.ZCertAttr
	(evecommon.HashAlgorithm)(0), // 6: org.lfedge.eve.common.HashAlgorithm
}
var file_certs_certs_proto_depIdxs = []int32{
	4, // 0: org.lfedge.eve.certs.ZControllerCert.certs:type_name -> org.lfedge.eve.certs.ZCert
	0, // 1: org.lfedge.eve.certs.ZCertMetaData.type:type_name -> org.lfedge.eve.certs.ZCertMetaDataType
	6, // 2: org.lfedge.eve.certs.ZCert.hashAlgo:type_name -> org.lfedge.eve.common.HashAlgorithm
	1, // 3: org.lfedge.eve.certs.ZCert.type:type_name -> org.lfedge.eve.certs.ZCertType
	5, // 4: org.lfedge.eve.certs.ZCert.attributes:type_name -> org.lfedge.eve.certs.ZCertAttr
	3, // 5: org.lfedge.eve.certs.ZCert.meta_data_items:type_name -> org.lfedge.eve.certs.ZCertMetaData
	6, // [6:6] is the sub-list for method output_type
	6, // [6:6] is the sub-list for method input_type
	6, // [6:6] is the sub-list for extension type_name
	6, // [6:6] is the sub-list for extension extendee
	0, // [0:6] is the sub-list for field type_name
}

func init() { file_certs_certs_proto_init() }
func file_certs_certs_proto_init() {
	if File_certs_certs_proto != nil {
		return
	}
	if !protoimpl.UnsafeEnabled {
		file_certs_certs_proto_msgTypes[0].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*ZControllerCert); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
		file_certs_certs_proto_msgTypes[1].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*ZCertMetaData); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
		file_certs_certs_proto_msgTypes[2].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*ZCert); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
		file_certs_certs_proto_msgTypes[3].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*ZCertAttr); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
	}
	type x struct{}
	out := protoimpl.TypeBuilder{
		File: protoimpl.DescBuilder{
			GoPackagePath: reflect.TypeOf(x{}).PkgPath(),
			RawDescriptor: file_certs_certs_proto_rawDesc,
			NumEnums:      2,
			NumMessages:   4,
			NumExtensions: 0,
			NumServices:   0,
		},
		GoTypes:           file_certs_certs_proto_goTypes,
		DependencyIndexes: file_certs_certs_proto_depIdxs,
		EnumInfos:         file_certs_certs_proto_enumTypes,
		MessageInfos:      file_certs_certs_proto_msgTypes,
	}.Build()
	File_certs_certs_proto = out.File
	file_certs_certs_proto_rawDesc = nil
	file_certs_certs_proto_goTypes = nil
	file_certs_certs_proto_depIdxs = nil
}
