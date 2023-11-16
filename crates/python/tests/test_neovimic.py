import unittest
import neovi_mic
from neovi_mic import IOBitMode
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
        self.assertEqual(self.mic.io.is_open(), False)
        self.mic.io.open()
        self.assertEqual(self.mic.io.is_open(), True)
        self.mic.io.close()
        self.assertEqual(self.mic.io.is_open(), False)

    def test_io_buzzer(self):
        self.mic.io.open()
        self.mic.io.set_bitmode_raw(IOBitMode.BuzzerMask.value | IOBitMode.Buzzer.value)
        self.assertEqual(self.mic.io.read_pins_raw(), IOBitMode.Buzzer.value)
        self.mic.io.set_bitmode_raw(IOBitMode.BuzzerMask.value)
        self.assertEqual(self.mic.io.read_pins_raw(), 0x00)
        self.mic.io.close()

if __name__ == '__main__':
    unittest.main()
