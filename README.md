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
