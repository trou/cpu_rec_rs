## cpu_rec_rs

Determine which CPU architecture is used in a binary file.
Example:

```
$ cpu_rec_rs /bin/bash /usr/lib/firmware/rtlwifi/rtl8821aefw*
Loading corpus from cpu_rec_corpus/*.corpus
----------------------------------------------------------------------------------------
                      File                       |     Range     | Detected Architecture
----------------------------------------------------------------------------------------
/bin/bash                                        | Whole file    | X86-64
/usr/lib/firmware/rtlwifi/rtl8821aefw_29.bin     | 0x3200-0x4400 | 8051
/usr/lib/firmware/rtlwifi/rtl8821aefw_29.bin     | 0x4600-0x5000 | 8051
/usr/lib/firmware/rtlwifi/rtl8821aefw_29.bin     | 0x6000-0x6600 | 8051
/usr/lib/firmware/rtlwifi/rtl8821aefw_29.bin     | 0x6600-0x6c00 | 8051
/usr/lib/firmware/rtlwifi/rtl8821aefw.bin        | Whole file    | 8051
/usr/lib/firmware/rtlwifi/rtl8821aefw_wowlan.bin | Whole file    | 8051
----------------------------------------------
```

Note: as the approach is based on statistics, false positives are definitely
possible. You should cross check with other sources and validate the results
with a disassembler.

In particular, small files are more prone to false positives, as well as smaller
sliding windows. Common false positives include:

* `xmos_xs2a`
* `NDS32`

### About

`cpu_rec_rs` is a Rust reimplementation of the original
[`cpu_res`](https://github.com/airbus-seclab/cpu_rec/). Why reimplement it?

* Performance
* Code simplification
* Rust practice


The original `cpu_rec` contains a lot of code necessary for experimenting and
updating the corpus. If you want to play with various settings for prediction,
please use `cpu_rec`. It also contains documentation and links to the theory
behind it ([SSTIC presentation](https://github.com/airbus-seclab/cpu_rec/blob/master/doc/cpu_rec_slides_english.pdf)).