hevc-reader
===========

Reader for HEVC / H265 bitstream syntax, written in Rust.

This is a pared down HEVC-targeted fork of the excellent [h264-reader](https://github.com/dholroyd/h264-reader) crate.

Documentation and tests are mostly untouched and out of date for now, and very little functionality is available except
the most basic NALs.


## Design goals

### Avoid copies

Parsing components accept partial data to avoid coping data
into intermediate buffers.  This is intended to support common cases like,

 - data embedded in MPEG-TS packets, where H265 data is interspersed with MPEG-TS header data
 - data being read from the network, where the data available at any instant may be incomplete

An alternative to accepting partial data would be to take a number of pieces of partial data

### Lazy parsing

The implementation should be written to defer parsing data structures until an accessor method is called.
This can mean saving parsing costs for callers that don't care about all the data.  It can be difficult to
apply this principal universally, so in some areas we don't bother and just 'eager parse' (particularly
structures defined bit-by-bit rather than byte-by-byte).
