import unittest
import neovi_mic
import time

class TestNeoVIMIC(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        import os; print(os.getpid())
        try:
            cls.mic = neovi_mic.find()[0]
        except IndexError:
            raise RuntimeError("ERROR: No NeoVI MIC2s found...")

    def test_finder(self):
        self.assertIsInstance(self.mic.get_serial_number(), str)
        self.assertIsInstance(self.mic.has_gps(), bool)

    def test_io_open_close(self):
        self.assertEqual(self.mic.io_is_open(), False)
        self.mic.io_open()
        self.assertEqual(self.mic.io_is_open(), True)
        self.mic.io_close()
        self.assertEqual(self.mic.io_is_open(), False)

    def test_io(self):
        # Open
        self.mic.io_open()
        self.assertEqual(self.mic.io_is_open(), True)

        # Test Buzzer
        self.mic.io_buzzer_enable(True)
        self.assertEqual(self.mic.io_buzzer_is_enabled(), True)
        time.sleep(0.1)
        self.mic.io_buzzer_enable(False)
        self.assertEqual(self.mic.io_buzzer_is_enabled(), False)

        # Test GPS LED
        self.mic.io_gpsled_enable(True)
        self.assertEqual(self.mic.io_gpsled_is_enabled(), True)
        time.sleep(0.1)
        self.mic.io_gpsled_enable(False)
        self.assertEqual(self.mic.io_gpsled_is_enabled(), False)

        # Test Button
        self.assertEqual(self.mic.io_button_is_pressed(), False)

        # Close
        self.mic.io_close()
        self.assertEqual(self.mic.io_is_open(), False)

    def test_audio(self):      
        try:
            # Turn on the buzzer so we have some sound to record
            self.mic.io_open()
            self.mic.io_buzzer_enable(True)
            self.mic.audio_start(44_100)
            time.sleep(0.5)
            self.mic.audio_stop()
            self.mic.audio_save("test.ogg")
        finally:
            # Always make sure we disable the buzzer, its annoying when left on.
            self.mic.io_buzzer_enable(False)
            self.mic.io_close()

    def test_gps_dms(self):
        dms = neovi_mic.GPSDMS()
        self.assertEqual(dms.degrees, 0)
        self.assertEqual(dms.minutes, 0)
        self.assertEqual(dms.seconds, 0)
        self.assertEqual(str(dms), "0° 0' 0\"")
        self.assertEqual(repr(dms), "<PyGPSDMS 0° 0' 0\">")

    def test_gps_sat_info(self):
        gps_sat_info = neovi_mic.GPSSatInfo()
        self.assertEqual(gps_sat_info.prn, 0)
        self.assertEqual(gps_sat_info.used, False)
        self.assertEqual(gps_sat_info.lock_time, 0)
        with self.assertRaises(ValueError):
            gps_sat_info.azimuth
        with self.assertRaises(ValueError):
            gps_sat_info.elevation
        with self.assertRaises(ValueError):
            gps_sat_info.snr
        gps_sat_info_str = f"prn: {gps_sat_info.prn}, used: false, azimuth: None, elevation: None, snr: None, lock_time: {gps_sat_info.lock_time}"
        self.assertEqual(str(gps_sat_info), gps_sat_info_str)
        self.assertEqual(repr(gps_sat_info), f"<PyGPSSatInfo {gps_sat_info_str}>")

    def test_gps(self):
        try:
            self.assertEqual(self.mic.gps_is_open(), False)
            self.mic.gps_open()
            self.assertEqual(self.mic.gps_is_open(), True)
            start = time.time()
            while (time.time() - start) < 1000:
                if self.mic.gps_has_lock():
                    info: neovi_mic.PyGPSInfo = self.mic.gps_info()
                    used_count = 0
                    for sat in info.satellites():
                        if sat.used:
                            used_count += 1
                    print(f"Sats: {used_count}/{len(info.satellites())}, Time: {info.current_time} lat: {str(info.latitude())}, long: {str(info.longitude())}")
                    print(f"{info.sog_kmh:.2f}Km/h\taltitude: {info.altitude}m\th_acc: {info.h_acc}m\tv_acc: {info.v_acc}m\n")
                else:
                    print("No GPS Lock")
                time.sleep(1)
            
        finally:
            pass #self.mic.gps_close()

if __name__ == '__main__':
    unittest.main()
