
```
$ hoppy container app get --id HbqFNk0KZjzYOcp
Error: failed to decode success response: error decoding response body: invalid type: null, expected a string at line 1 column 606
```
When using --debug I'd then expect to see the returned json:
```
$ hoppy container app get --id HbqFNk0KZjzYOcp --debug
>> GET https://api.bunny.net/mc/apps/HbqFNk0KZjzYOcp
Error: failed to decode success response: error decoding response body: invalid type: null, expected a string at line 1 column 606
```
