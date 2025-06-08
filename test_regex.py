import re

# Test the config content
config_content = """AvdId=Pixel_9
PlayStore.enabled=no
abi.type=arm64-v8a
avd.id=<build>
avd.ini.displayname=Pixel 9
avd.ini.encoding=UTF-8
avd.name=<build>
image.sysdir.1=system-images/android-36/google_apis/arm64-v8a/
target=android-36
"""

# Test regex patterns
pattern1 = r"image\.sysdir\.1=system-images/android-(\d+)/?"
pattern2 = r"target=android-(\d+)"

match1 = re.search(pattern1, config_content)
match2 = re.search(pattern2, config_content)

print("Testing regex patterns:")
print(f"Pattern 1 ({pattern1}): {match1.group(1) if match1 else 'No match'}")
print(f"Pattern 2 ({pattern2}): {match2.group(1) if match2 else 'No match'}")

# Test line-by-line
for line in config_content.split('\n'):
    if line.startswith("image.sysdir.1=system-images/android-"):
        print(f"Found image.sysdir.1 line: {line}")
        if "android-" in line:
            start = line.find("android-") + 8
            after_android = line[start:]
            if "/" in after_android:
                end = after_android.find("/")
                api = after_android[:end]
                print(f"Extracted API: {api}")
    elif line.startswith("target=android-"):
        print(f"Found target line: {line}")
        if "android-" in line:
            start = line.find("android-") + 8
            api = line[start:]
            print(f"Extracted API: {api}")
