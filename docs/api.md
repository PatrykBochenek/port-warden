# API Reference

This page documents the public API of the `portly` package. The Python
functions are thin wrappers over a native Rust core; see the
[Development](development.md) page for how the two layers fit together.

## Exceptions

All errors raised by portly derive from `PortlyError`. The concrete
subclasses are also subclasses of the builtin `OSError` / `PermissionError`,
so existing handlers keep working.

::: portly.PortlyError

::: portly.PortlyPortError

::: portly.PortlyPermissionError

## Functions

::: portly.is_available

::: portly.find_free

::: portly.find_free_in_range

::: portly.wait_until_free

::: portly.wait_for_server

::: portly.get_info

::: portly.kill

::: portly.scan