import unittest
import neovi_mic
import time

class TestNeoVIMIC(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        import os; print(os.getpid())
        cls.mic = neovi_mic.find()[0]

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
    


if __name__ == '__main__':
    unittest.main()
