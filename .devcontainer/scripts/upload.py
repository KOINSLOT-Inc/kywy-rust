# SPDX-FileCopyrightText: 2025 2023 - 2025 KOINSLOT, Inc.
#
# SPDX-License-Identifier: GPL-3.0-or-later

import sys
import os
import time
import shutil
import platform
import requests
import tempfile
import serial
import serial.tools.list_ports
import pyudev

def find_rp2040_serial():
    ports = serial.tools.list_ports.comports()
    for port in ports:
        if port.manufacturer and "koinslot" in port.manufacturer.lower():
            print(f"[DEBUG] Found Koinslot RP2040 serial at {port.device}")
            return port.device
    return None

def touch_1200_baud(port):
    try:
        ser = serial.Serial(port, baudrate=1200)
        ser.close()
        print(f"[DEBUG] Sent 1200 baud to {port}")
    except Exception as e:
        print(f"[ERROR] Failed to touch 1200 baud on {port}: {e}")

def check_rp2040_block(device):
    try:
        return device.get('ID_FS_LABEL', '') == 'RPI-RP2'
    except Exception:
        return False

def mount_or_find_mount(device):
    device_node = device.device_node
    mounts = {}
    with open('/proc/mounts') as f:
        for line in f:
            parts = line.split()
            if len(parts) >= 2:
                mounts[parts[1]] = parts[0]
    for mount_point, mount_device in mounts.items():
        if mount_device == device_node:
            return mount_point
    return None

def find_rp2040_drive(timeout=100):
    context = pyudev.Context()
    for device in context.list_devices(subsystem='block'):
        if check_rp2040_block(device):
            return mount_or_find_mount(device)

    monitor = pyudev.Monitor.from_netlink(context)
    monitor.filter_by('block')
    monitor.start()
    start = time.time()
    while time.time() - start < timeout:
        device = monitor.poll(timeout=0.5)
        if device and device.action == 'add' and check_rp2040_block(device):
            return mount_or_find_mount(device)
    return None

def check_rp2040_drive(path):
    return os.path.exists(os.path.join(path, "INFO_UF2.TXT"))

def download_uf2(url):
    if url.startswith("file://"):
        path = url[7:]
        if not os.path.isfile(path):
            raise Exception(f"Local file not found: {path}")
        return path
    resp = requests.get(url)
    if resp.status_code != 200:
        raise Exception(f"Failed to download UF2 from {url}")
    tmp = tempfile.NamedTemporaryFile(delete=False, suffix=".uf2")
    tmp.write(resp.content)
    tmp.close()
    print(f"[DEBUG] Downloaded to {tmp.name}")
    return tmp.name

def flash_file(uf2_path):
    print("[INFO] Preparing UF2 source...")
    uf2_local = download_uf2(uf2_path)

    print("[INFO] Attempting to find or trigger RP2040...")
    drive = find_rp2040_drive(timeout=2)
    if not drive:
        port = find_rp2040_serial()
        if not port:
            print("[ERROR] No RP2040 serial device found.")
            return
        print(f"[INFO] Resetting board via 1200 baud on {port}")
        touch_1200_baud(port)
        time.sleep(0.5)
        touch_1200_baud(port)
        print("[INFO] Waiting for RPI-RP2 mount...")
        time.sleep(5)
        drive = find_rp2040_drive(timeout=100)
        if not drive:
            print("[ERROR] RP2040 drive did not appear.")
            return

    if not check_rp2040_drive(drive):
        print(f"[ERROR] {drive} is not a valid RPI-RP2 mount.")
        return

    dest = os.path.join(drive, os.path.basename(uf2_local))
    print(f"[INFO] Copying {uf2_local} → {dest}")
    shutil.copy(uf2_local, dest)
    print("[INFO] Flash complete.")

def main():
    if len(sys.argv) < 2:
        print("Usage: python flash_uf2.py <file1.uf2> [file2.uf2 ...]")
        return
    for arg in sys.argv[1:]:
        try:
            flash_file(arg)
        except Exception as e:
            print(f"[ERROR] {e}")

if __name__ == "__main__":
    main()
