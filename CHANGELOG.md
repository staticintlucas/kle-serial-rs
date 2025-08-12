# Change Log

## [v0.3.2](https://github.com/staticintlucas/kle-serial-rs/releases/tag/v0.3.2)

### Changes

* Update CI workflow to use lock file for MSRV testing
* Removed pre-commit hooks
* Improvements to parts of the documentation
* Added cargo-rdme to keep README in sync with documetation

### Fixes

* Fix error in key position for keys placed to the right of a stepped key

## [v0.3.1](https://github.com/staticintlucas/kle-serial-rs/releases/tag/v0.3.1)

### New

* Add `PartialEq` implementations for public types

### Changes

* Update CI workflow
* Update pre-commit hooks

### Fixes

* Fix various Clippy warnings and lints

## [v0.3.0](https://github.com/staticintlucas/kle-serial-rs/releases/tag/v0.3.0)

### New

* Add generic type parameter to structs for floating point data
  * This allows for deserialising KLE files into any type that implements `num_traits::real::Real`
  * The default is `f64`, but can be changed by specifying the type parameter when deserialising
* Add `f32` and `f64` modules to re-export the generic types with the `f32` and `f64` type
  parameters respectively
  * For example, `kle_serial::Keyboard<f32>` and `kle_serial::f32::Keyboard` are equivalent

## [v0.2.2](https://github.com/staticintlucas/kle-serial-rs/releases/tag/v0.2.2)

### Fixes

* Fixed bug exposed by serde v1.0.182 causing layouts with no metadata to deserialise incorrectly

## [v0.2.1](https://github.com/staticintlucas/kle-serial-rs/releases/tag/v0.2.1)

### Fixes

* Fix documentation build failure on [docs.rs]

[docs.rs]: https://docs.rs/kle-serial/0.2.1/kle_serial/

## [v0.2.0](https://github.com/staticintlucas/kle-serial-rs/releases/tag/v0.2.0)

### New

* Added a `KeyIterator` type to allow direct deserialisation of the layout's `Key`s

### Changes

* Documentation updates and improvements

## [v0.1.1](https://github.com/staticintlucas/kle-serial-rs/releases/tag/v0.1.1)

### Changes

* Tweaked formatting in examples
* Fixed README formatting

### Fixes

* Fix bug in handling of legend alignment in certain use cases

## [v0.1.0](https://github.com/staticintlucas/kle-serial-rs/releases/tag/v0.1.0)

### New

* Initial release
* Full support for deserialising KLE JSON files
