# decoder

Simple utility to help with packet captures.

Accepts any hex string as it's input, stripping away whitespace, commas, and '0x' strings.
Collects multiple lines into one buffer as long as they end in `,`.

Reads input until EOF.

## Example

Input:

```
0x21, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x61, 0xde,
0x39, 0x7d, 0x31, 0x4f, 0x69, 0x40, 0x86, 0xd2,
0xcb, 0x38, 0x33, 0x94, 0x8b, 0xcc, 0x0a, 0x0b,
0x74, 0x65, 0x73, 0x74, 0x2d, 0x73, 0x74, 0x72,
0x65, 0x61, 0x6d, 0x10, 0x00
```

Output:

```
Full { bytes: 37, package: Package { authentication: None, correlation_id: Uuid("61de397d-314f-6940-86d2-cb3833948bcc"), message: Unsupported(192, [10, 11, 116, 101, 115, 116, 45, 115, 116, 114, 101, 97, 109, 16, 0]) } }
```
