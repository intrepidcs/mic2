import pymic2
import time

if __name__ == "__main__":
    devices = pymic2.find()
    print(f"Found {len(devices)} device(s)...")
    
    for device in devices:
        mic = device
        if mic.has_gps():
            break
    if not mic.has_gps():
        raise RuntimeError("No GPS found...")

    mic.gps_open()
    while not mic.gps_has_lock():
        print("Waiting for GPS lock...")
        time.sleep(1)
    while True:
        info = mic.gps_info()
        used_count = 0
        for sat in info.satellites():
            if sat.used:
                used_count += 1
        mph = info.sog_kmh / 1.609344
        print(f"Sats: {used_count}/{len(info.satellites())}, Time: {info.current_time} lat: {str(info.latitude())}, long: {str(info.longitude())}")
        print(f"{mph:.2f}mph ({info.sog_kmh:.2f}Km/h)\taltitude: {info.altitude}m\th_acc: {info.h_acc}m\tv_acc: {info.v_acc}m cog: {info.cog}\t nav_sat: {info.nav_stat()}\n")
        time.sleep(1)
