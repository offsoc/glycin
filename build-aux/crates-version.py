#!/usr/bin/python3

import tomllib

data = tomllib.load(open('Cargo.toml', 'rb'))

print(data['workspace']['package']['version'], end='')
