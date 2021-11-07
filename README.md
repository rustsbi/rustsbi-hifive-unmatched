# RustSBI 在 HiFive Unmatched 主板的支持软件

这个项目的目的是在SiFive [HiFive Unmatched](https://www.sifive.com/boards/hifive-unmatched)主板上支持RustSBI。
RustSBI是一个引导程序环境；主板上电时，RustSBI将会先行启动，而后，它将会找到一个可引导的操作系统，引导启动这个操作系统。
在启动后，RustSBI仍然常驻后台，提供操作系统需要的功能。
RustSBI的设计完全符合RISC-V SBI规范标准，只要支持此标准的操作系统，都可以使用RustSBI引导启动。

## 对大小核设计的支持方案

HiFive Unmatched主板板载SiFive Freedom U740处理器。FU740是异构的多核处理器，它总共有五个核。
它的五个核分别为四个U74应用处理器内核，以及一个S7嵌入式处理器内核。

作为RustSBI的软件实现开发者，我们根据S7管理小核的用途，大致有两种实现RustSBI的方法。
一种是独占S7管理小核，那么四个U74应用大核就留给操作系统使用。
在一些应用场景中，即使管理小核的性能不如应用大核，它也能辅助运行一些后台应用，从而更合理地安排需要的计算和I/O任务执行工作。
此时，RustSBI可以实现为后台运行的应用，这样将五个核——包括应用大核和管理小核——都提供给操作系统。

由于FU740芯片的异构设计，RustSBI在HiFive Unmatched上将提供一个配置选项，来决定RustSBI是否应该将管理小核提供给操作系统。
根据需要运行的操作系统需求，首次启动或重新烧录RustSBI固件后，您可以手动选择配置选项，来决定是否提供管理小核给操作系统。

## 开发日程

- 11月7日：了解FU740处理器，阅读文档，编写方案
- 11月8-14日：实现RustSBI与SBI标准的大部分功能
- 11月15-21日：在Unmatched上调试rCore和Linux
- 11月22日-12月6日：根据用户反馈集中修理bugs，形成详细的使用文档，提供一份汇报报告

## 有用的链接

- HiFive Unmatched 入门指南（中文）1.4版 [PDF](https://sifive.cdn.prismic.io/sifive/b9376339-5d60-45c9-8280-58fd0557c2f0_hifive-unmatched-gsg-v1p4_ZH.pdf)
- SiFive FU740-000 Manual v1p3 [PDF](https://sifive.cdn.prismic.io/sifive/de1491e5-077c-461d-9605-e8a0ce57337d_fu740-c000-manual-v1p3.pdf)

## 实现方法（草稿）

- 可以试试看开机的时候按下F12？这就和x86的生态有点像了。首次启动默认不提供管理小核，需要的话重启，进SBI界面（串口？），打开。——@luojia65 2021/11/7
- U74和S7都是64位的，所以都用RV64去编译就可以了。——@luojia65 2021/11/7
