# sensors-zabbix

## Usage
```
% sensors-discovery
{
  "data": [
    {
      "{#DEVICE_NAME}": "coretemp",
      "{#SENSOR_NAME}": "temp2"
    },
    {
      "{#DEVICE_NAME}": "coretemp",
      "{#SENSOR_NAME}": "temp5"
    },
    {
      "{#DEVICE_NAME}": "coretemp",
      "{#SENSOR_NAME}": "temp3"
    },
    {
      "{#DEVICE_NAME}": "coretemp",
      "{#SENSOR_NAME}": "temp1"
    },
    {
      "{#DEVICE_NAME}": "coretemp",
      "{#SENSOR_NAME}": "temp4"
    }
  ]
}
% sensors-get coretemp temp1_input
37
```
