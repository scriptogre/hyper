import json

with open('rust/tests/kitchen_sink.hyper', 'r', encoding='utf-8') as f:
    text = f.read()

# Convert to UTF-16 code units so we can slice by UTF-16 offsets
encoded = text.encode('utf-16-le')

with open('rust/tests/kitchen_sink.expected.json', 'r') as f:
    data = json.load(f)

print("--- PYTHON INJECTIONS ---")
for inj in data.get('injections', []):
    if inj['type'] == 'python':
        start = inj['start']
        end = inj['end']
        # UTF-16-LE uses 2 bytes per code unit
        extracted_utf16 = encoded[start*2 : end*2]
        extracted_text = extracted_utf16.decode('utf-16-le')
        print(f"[{start}:{end}] {repr(extracted_text)}")
