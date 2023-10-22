import unittest
import neovi_mic

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


if __name__ == '__main__':
    unittest.main()
