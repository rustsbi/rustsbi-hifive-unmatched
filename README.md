# RustSBI support software for HiFive Unmatched boards

The purpose of this project is to support RustSBI on SiFive [HiFive Unmatched](https://www.sifive.com/boards/hifive-unmatched) boards.
RustSBI is a bootloader environment; when the motherboard is powered up, RustSBI will start first, then it will find a bootable operating system and boot it.
After startup, RustSBI still resides in the background, providing the functionality required by the operating system.
The design of RustSBI fully complies with the RISC-V SBI specification standard. As long as the operating system supports this standard, RustSBI can be used to boot.

## Compile and Run

This project uses the xtask framework and can be compiled with the following instructions:

```shell
cargo image
```

(If the --release parameter is added, it means that the release version without debugging symbols is compiled)

At this time, the compilation produces an elf file and an img image. Note that the generated intermediate data bin file cannot be directly used for burning.

Use the following operations to burn the img format image to the sd card partition. (Dangerous! Data must be backed up first)

```shell
sudo dd if=target/sd-card-partition-2.img of=\\?\Device\Harddisk????\Partition2 --progress
```

After the burning is complete, you can use RustSBI to boot.

## Rust Version

Compiling this project requires at least the Rust version of `rustc 1.59.0-nightly (c5ecc1570 2021-12-15)`.

## Documentation

Please refer to the [Project Wiki](https://github.com/rustsbi/rustsbi-hifive-unmatched/wiki) for complete documentation.

The parts of the project that still need to be perfected are also documented in [here](https://github.com/rustsbi/rustsbi-hifive-unmatched/wiki/Next…).

## Support scheme for large and small core design

The HiFive Unmatched motherboard has a SiFive Freedom U740 processor onboard. The FU740 is a heterogeneous multi-core processor, it has a total of five cores.
Its five cores are four U74 application processor cores and one S7 embedded processor core.

As developers of the software implementation of RustSBI, we noticed that the S7 management corelet will have a wide range of uses.
Therefore, RustSBI does not block any cores on HiFive Unmatched for the operating system to choose and use.

## useful links

- HiFive Unmatched Getting Started Guide (Chinese) Version 1.4 [PDF](https://sifive.cdn.prismic.io/sifive/b9376339-5d60-45c9-8280-58fd0557c2f0_hifive-unmatched-gsg-v1p4_EN.pdf)
- SiFive FU740-000 Manual v1p3 [PDF](https://sifive.cdn.prismic.io/sifive/de1491e5-077c-461d-9605-e8a0ce57337d_fu740-c000-manual-v1p3.pdf)

## Command Line

View assembly code

```
cargo asm
```

---
---

# RustSBI 在 HiFive Unmatched 主板的支持软件

这个项目的目的是在SiFive [HiFive Unmatched](https://www.sifive.com/boards/hifive-unmatched)主板上支持RustSBI。
RustSBI是一个引导程序环境；主板上电时，RustSBI将会先行启动，而后，它将会找到一个可引导的操作系统，引导启动这个操作系统。
在启动后，RustSBI仍然常驻后台，提供操作系统需要的功能。
RustSBI的设计完全符合RISC-V SBI规范标准，只要支持此标准的操作系统，都可以使用RustSBI引导启动。

## 编译和运行

这个项目使用xtask框架，可以使用以下指令来编译：

```shell
cargo image
```

（如果增加--release参数，说明编译的是不带调试符号的release版本）

这时候编译产生一个elf文件和一个img镜像。注意，产生的中间数据bin文件不可以直接用于烧录。

使用以下操作来烧录img格式的镜像到sd卡分区。（危险！必须先备份数据）

```shell
sudo dd if=target/sd-card-partition-2.img of=\\?\Device\Harddisk????\Partition2 --progress
```

烧录完成后，就可以使用RustSBI引导启动了。

## Rust版本

编译这个项目至少需要`rustc 1.59.0-nightly (c5ecc1570 2021-12-15)`的Rust版本。

## 文档

请参考[项目Wiki](https://github.com/rustsbi/rustsbi-hifive-unmatched/wiki)来获取完整的文档。

项目中仍然需要完善的部分也被记录在文档中，详见[这里](https://github.com/rustsbi/rustsbi-hifive-unmatched/wiki/接下来要做……)。

## 对大小核设计的支持方案

HiFive Unmatched主板板载SiFive Freedom U740处理器。FU740是异构的多核处理器，它总共有五个核。
它的五个核分别为四个U74应用处理器内核，以及一个S7嵌入式处理器内核。

作为RustSBI的软件实现开发者，我们注意到S7管理小核将有广泛的用途。
因此，RustSBI在HiFive Unmatched上不屏蔽任何的核，以供操作系统选择和使用。

## 有用的链接

- HiFive Unmatched 入门指南（中文）1.4版 [PDF](https://sifive.cdn.prismic.io/sifive/b9376339-5d60-45c9-8280-58fd0557c2f0_hifive-unmatched-gsg-v1p4_ZH.pdf)
- SiFive FU740-000 Manual v1p3 [PDF](https://sifive.cdn.prismic.io/sifive/de1491e5-077c-461d-9605-e8a0ce57337d_fu740-c000-manual-v1p3.pdf)

## 命令行

查看汇编代码

```
cargo asm
```
