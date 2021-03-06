#!/bin/bash

set -e

if [ -z "${PREFIX}" ]; then
    echo "No PREFIX specified. Dont know where to install rd files!"
    echo ""
    echo "e.g. $ PREFIX=~/myrd ./install.sh"
    echo "     This command will install rd files and directories to ~/myrd "
    echo "     rd related files will be stored in ~/myrd/bin, ~/myrd/lib and ~/myrd/share"
    exit 1
fi

if [ -d "${PREFIX}" ]; then
    echo "Installing rd to: ${PREFIX}"
    echo "NOTE: ${PREFIX}/bin, ${PREFIX}/lib, ${PREFIX}/share will be populated with rd files..."
else
    echo "'${PREFIX}' does not exist. Trying to create it"
    install -v -d "${PREFIX}"
fi

cargo install --locked --force --path . --root "${PREFIX}"

echo "Installing additional files and directories"
install -v -d "${PREFIX}/share/rd"
install -v -m 0644 -C target/share/rd/rd_page_64 "${PREFIX}/share/rd"
install -v -m 0644 -C target/share/rd/rd_page_64_replay "${PREFIX}/share/rd"
install -v -m 0644 -C target/share/rd/rd_page_32 "${PREFIX}/share/rd"
install -v -m 0644 -C target/share/rd/rd_page_32_replay "${PREFIX}/share/rd"

install -v -d "${PREFIX}/share/rd/src/preload"
install -v -m 0644 -C target/share/rd/src/preload/syscall_hook.S "${PREFIX}/share/rd/src/preload"
install -v -m 0644 -C target/share/rd/src/preload/syscallbuf.c "${PREFIX}/share/rd/src/preload"
install -v -m 0644 -C target/share/rd/src/preload/raw_syscall.S "${PREFIX}/share/rd/src/preload"
install -v -m 0644 -C target/share/rd/src/preload/breakpoint_table.S "${PREFIX}/share/rd/src/preload"
install -v -m 0644 -C target/share/rd/src/preload/overrides.c "${PREFIX}/share/rd/src/preload"
install -v -m 0644 -C target/share/rd/src/preload/preload_interface.h "${PREFIX}/share/rd/src/preload"
install -v -m 0644 -C target/share/rd/src/preload/syscallbuf.h "${PREFIX}/share/rd/src/preload"

install -v -d "${PREFIX}/lib/rd"
install -v -m 0644 -C target/lib/rd/librdpreload.so "${PREFIX}/lib/rd"
install -v -m 0644 -C target/lib/rd/librdpreload_32.so "${PREFIX}/lib/rd"

install -v -d "${PREFIX}/bin"
install -v -m 0755 -C target/bin/rd_exec_stub "${PREFIX}/bin"
install -v -m 0755 -C target/bin/rd_exec_stub_32 "${PREFIX}/bin"

echo "Done"
