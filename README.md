# IoT Penetration Testing: Security analysis of a car dongle 
Proof of concept for hack on AutoPi found during bachelor thesis ([link](https://www.diva-portal.org/smash/record.jsf?pid=diva2%3A1334244), CVE-2019-12941).

## Vulnerability
The Raspberry Pi which the AutoPi is built upon, has a unique 8 character hex serial number. This number is md5 hashed into a 32 character hex string, also known as the “dongle id“, “unit id” or “minion id” [[row 9]](https://github.com/autopi-io/autopi-core/blob/3507b5ff420c9e7af3aa88b0b1cf4b68e677b36a/src/salt/base/state/minion/install.sls). The dongle id is a unique identifier of the AutoPi dongle and the first 6 bytes are used as wifi password while the last 6 bytes are used as wifi SSID. This means that one can deduce the the wifi password from the broadcasted SSID. Root access is given if connected to the AutoPi dongle via wifi. 

This program create a wordlist of all possible hashes sorted by the last 6 bytes (the SSID). It is implemented using a external sorting algorithm and will create a file of size 64GB.

```
USAGE:
    thesis-rust.exe [OPTIONS]

OPTIONS:
    -o, --output <PATH>         Name of output file. [default: "list"]
    -s, --start <u64>           Start value of serial number. [default: 0]
    -e, --end <u64>             End value of serial number. [default: 0xffff_ffff]
    -b, --buffer_size <u64>     ~Buffer size in bytes. [default: 4_294_967_296]
    -p, --print_amount <u64>    Print status message every "print_amount" iteration. [default: 200_000_000]
    -t, --threads <usize>       ~Max amount of threads. [default: (# logical CPUs)]
```
