# Changelog

All notable changes to the ROS bag reader and writer project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2023-12-26

### Added

- Initial implementation of the ROS bag writer in Rust, including:
  - Basic structure setup for the `Writer` struct.
  - Implementation of the `add_connection` and `write` methods.
- Initial setup of the ROS bag reader in Rust, including:
  - Basic structure for the `Reader` struct.
  - Implementation of the `open` method for the `Reader` struct to initialize storage based on metadata.

### Changed

- Refinement and updates to error handling mechanisms in both reader and writer components.
- Various optimizations and code cleanups to improve performance and readability.
- Add Drop for `Writer` to create metadata.yaml when Writer dropped.

## [0.1.0] - 2023-12-23

- Initial release of the project.
