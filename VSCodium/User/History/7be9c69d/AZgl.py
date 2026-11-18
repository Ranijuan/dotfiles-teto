path = "/home/ranijuan/ajustes-hypr.py"
with open(path, encoding="utf-8") as f:
    content = f.read()

old = '''        for k, _, _ in ALL:
            v = vals.get(k)
            if v and (v != defaults[k] or k in self.loaded):
                out.append('  %s = "%s",' % (k, v.replace('"', "")))'''

new = '''        for k, _, _ in APPS:
            v = vals.get(k)
            if v and (v != defaults[k] or k in self.loaded):
                out.append('  %s = "%s",' % (k, v.replace('"', "")))
        for k, _, _ in KEYS:
            v = vals.get(k)
            if v:
                out.append('  %s = "%s",' % (k, v.replace('"', "")))'''

content = content.replace(old, new, 1)
with open(path, "w", encoding="utf-8") as f:
    f.write(content)
print("listo")