# Bitvmx Workspace
This workspace is designed to provide a comprehensive development environment for working with the entire BitVMX ecosystem. By organizing the related libraries as submodules under the rust-bitvmx-client repository, developers can seamlessly work across all BitVMX components while maintaining proper version control and dependency management.

The rust-bitvmx-client serves as the main entry point for the BitVMX libraries. This modular approach allows for:

- Unified development across all BitVMX components
- Consistent version management of interdependent libraries
- Easy synchronization of updates across the ecosystem

The workspace structure enables developers to make changes to any of the BitVMX libraries while testing the effects in the client implementation, ensuring a cohesive development experience.

## Installation
Clone the repository and initialize the submodules:
```bash
git clone --recurse-submodules git@github.com:FairgateLabs/rust-bitvmx-workspace.git
```

OR manually initialize the submodules (if you already cloned the repo without the `--recurse-submodules` option):

```bash
git clone git@github.com:FairgateLabs/rust-bitvmx-workspace.git
git submodule init
git submodule update --remote --checkout
```
