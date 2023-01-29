# Co2mon-rs

**Co2mon-rs** is a program to read CO2, Temperature, and Humidity data from a CO2-Monitor device distributed by various suppliers. It reads from a USB HID interface and publishes measurements to an MQTT topic.

### Run

Set two ENV variables containing your MQTT credentials, like so:
```
export MQTT_USER=<username>
export MQTT_PASS=<password>
```
Start the service by running
```
cargo run -- --mqtt_broker <host> --mqtt_topic <topic>
```

### References:

The reverse engineering of the device and the original software version is
done by: Henryk Plötz, (2015):
https://hackaday.io/project/5301-reverse-engineering-a-low-cost-usb-co-monitor/log/17909-all-your-base-are-belong-to-us

Python implementation:
https://sourceforge.net/projects/minimon/

Some later software with bug fixes is by heinemml:
https://github.com/heinemml/CO2Meter

Device documentation:
http://co2meters.com/Documentation/AppNotes/AN146-RAD-0401-serial-communication.pdf

An explanation of how an NDIR CO2 Sensor works:
https://www.co2meter.com/blogs/news/6010192-how-does-an-ndir-co2-sensor-work
