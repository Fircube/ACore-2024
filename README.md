# ACore-2024
---

A simple RISC-V monolithic kernel based on  [rCore-Tutorial-v3](https://github.com/rcore-os/rCore-Tutorial-v3).

## Implementation

- Bootloader
- Allocator
  - Buddy allocator
  - Frame allocator
  - SLUB (may have bugs)
- Page table with SV39
- Console
- Message & data transfer
- Process
  - Process loading
  - Syscall
  - Process manager
  - Scheduler
- an userspace interactive shell


## Run

To run this project, you could use the following commands:

``` bash
$ git clone git@github.com:Fircube/ACore-2024.git
$ cd Acore-2024/usr
$ make build
$ cd ../kernel
$ make run
```
