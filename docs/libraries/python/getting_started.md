# Getting Started with IOTA Client Python binding

## Installation
Easiest way how to get python binding up and running is to leverage pre-built python libraries for linux/macos/windows that can be installed to your python environment (3.6+) via `pip`. The binding is automagically generated using github [actions](https://github.com/iotaledger/iota.rs/actions/workflows/python_binding_publish.yml).

The latest artifacts for major python versions can be also grabbed using [nighly.link service](https://nightly.link/iotaledger/iota.rs/workflows/python_binding_publish/dev). Download zip file for the given os and pyversion, unpack wheel file (`.whl`) and install it via `pip`:

```bash
pip install <wheel_file>
```

Once it has been properly installed you can double check it using `pip`:
```bash
pip list
```

You should see the similar output:
```plaintext
Package                    Version
-------------------------- -------
iota-core-python           0.2.0a3
```
Once installed in the given python environment you are all set and can start hacking using python binding!

## Usage

```python
import iota_client
print(iota_client.__doc__)
print(dir(iota_client))
```
