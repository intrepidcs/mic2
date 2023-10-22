import unittest
import neovi_mic
import time

class TestNeoVIMIC(unittest.TestCase):
    def test_finder(self):
        for mic in neovi_mic.find():
            print(mic, mic.get_serial_number(), mic.has_gps())
            mic: neovi_mic.NeoVIMIC = mic
            ftdi_device: neovi_mic.UsbDeviceInfo = mic.get_ftdi_device()
            print(ftdi_device, 
                hex(ftdi_device.vendor_id),
                hex(ftdi_device.product_id),
                hex(ftdi_device.address),
                hex(ftdi_device.bus_number)
            )

    def test_io_open_close(self):
        for mic in neovi_mic.find():
            io: neovi_mic.IODevice = mic.get_io_device()
            self.assertEqual(io.is_open(), False)
            io.open()
            self.assertEqual(io.is_open(), True)
            io.close()
            self.assertEqual(io.is_open(), False)

    def test_io_buzzer(self):
        for mic in neovi_mic.find():
            io: neovi_mic.IODevice = mic.get_io_device()
            io.open()
            io.set_bitmode_raw(0x51)
            self.assertEqual(io.read_pins_raw(), 0x01)
            io.set_bitmode_raw(0x50)
            self.assertEqual(io.read_pins_raw(), 0x00)
            io.close()
    
    def test_io_buzzer(self):
        for mic in neovi_mic.find():
            io: neovi_mic.IODevice = mic.get_io_device()
            io.open()
            io.set_bitmode_raw(0x54)
            self.assertEqual(io.read_pins_raw(), 0x04)
            io.set_bitmode_raw(0x50)
            self.assertEqual(io.read_pins_raw(), 0x00)
            io.close()

if __name__ == '__main__':
    unittest.main()
