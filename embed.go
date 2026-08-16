package hnm

import "embed"

// TemplateFS holds the same files the Rust crate embeds from templates/.
//
//go:embed all:templates
var TemplateFS embed.FS

// HarnessSchema describes the contract layout consumed via ctxl.
//
//go:embed schema/harness.json
var HarnessSchema []byte
