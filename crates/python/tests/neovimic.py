import unittest
import neovi_mic
import time

class TestNeoVIMIC(unittest.TestCase):
    @classmethod
    def setupClass(cls):
        cls.mic = neovi_mic.find()[0]

    def test_finder(self):
        self.assertIsInstance(self.mic.get_serial_number(), str)
        self.assertIsInstance(self.mic.has_gps(), bool)

    def test_io_open_close(self):
        self.assertEqual(self.mic.io.is_open(), False)
        self.mic.io.open()
        self.assertEqual(self.mic.io.is_open(), True)
        self.mic.io.close()
        self.assertEqual(self.mic.io.is_open(), False)

    def test_io_buzzer(self):
        self.mic.io.open()
        self.mic.io.set_bitmode_raw(0x51)
        self.assertEqual(self.mic.io.read_pins_raw(), 0x01)
        self.mic.io.set_bitmode_raw(0x50)
        self.assertEqual(self.mic.io.read_pins_raw(), 0x00)
        self.mic.io.close()
    
    def test_io_buzzer(self):
        self.mic.io.open()
        self.mic.io.set_bitmode_raw(0x54)
        self.assertEqual(self.mic.io.read_pins_raw(), 0x04)
        self.mic.io.set_bitmode_raw(0x50)
        self.assertEqual(self.mic.io.read_pins_raw(), 0x00)
        self.mic.io.close()

if __name__ == '__main__':
    unittest.main()
