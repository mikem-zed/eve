module github.com/lf-edge/eve/pkg/measure-config

go 1.20

require (
	github.com/canonical/tcglog-parser v0.0.0-20221030122402-0bee1fbba1d9
	github.com/google/go-tpm v0.3.3
)

require (
	github.com/canonical/go-efilib v0.3.1-0.20220314143719-95d50e8afc82 // indirect
	github.com/canonical/go-sp800.108-kdf v0.0.0-20210315104021-ead800bbf9a0 // indirect
	github.com/canonical/go-tpm2 v0.1.0 // indirect
	golang.org/x/sys v0.0.0-20210908233432-aa78b53d3365 // indirect
	golang.org/x/xerrors v0.0.0-20200804184101-5ec99f83aff1 // indirect
)

replace github.com/canonical/tcglog-parser v0.0.0-20221030122402-0bee1fbba1d9 => github.com/mikem-zed/tcglog-parser v0.0.0-20230413004639-7ae5145e8511
