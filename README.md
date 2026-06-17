# win-etw-manifest

Parser and Rust structures to work with Microsoft Windows instrumentation manifests (XML ETW (Event Tracing for Windows) provider manifests).

See the [schema definition](https://learn.microsoft.com/en-us/windows/win32/wes/eventmanifestschema-schema) provided by Microsoft.

## Current State

This Crate is implemented in an adhoc fashion, supporting the needs of the developers.
Currently the manifest schema isn't completely implemented.

It was build targeting manifests exported with tools relying on the functionality of [PerfView](https://github.com/microsoft/perfview/tree/main)'s [manifest extraction implementation](https://github.com/microsoft/perfview/blob/1a18deaf3f5353ada837e5c7942abb50e0f1f230/src/TraceEvent/RegisteredTraceEventParser.cs)
(like [EtwExplorer](https://github.com/zodiacon/EtwExplorer) by Pavel Yosifovich).

Feel free to contribute.
