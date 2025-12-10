import sys
import json
from glob import glob

folder = sys.argv[1]

out_json = {}
out_json["polyfills"] = []
out_json["polyfillAliases"] = {}
out_json["version"] = "add version here"

for polyfill_folder in glob(folder + "/*"):
    polyfill = polyfill_folder.replace(folder + "/", "")

    if polyfill == "aliases.json":
        continue
    if polyfill.startswith("_"):
        continue

    if ".~" in polyfill:
        index = polyfill.index(".~")
        polyfill = polyfill[:index]

    f = open(polyfill_folder + "/meta.json", "r")
    meta = json.loads(f.read())
    f.close()

    if "aliases" in meta:
        for alias in meta["aliases"]:
            if alias.startswith("dom") or alias.startswith("es") or alias == "default":
                if alias not in out_json["polyfillAliases"]:
                    out_json["polyfillAliases"][alias] = []

                out_json["polyfillAliases"][alias].append(polyfill)

    if polyfill not in out_json["polyfills"]:
        out_json["polyfills"].append(polyfill)


out_json["polyfills"].sort()
for alias in out_json["polyfillAliases"]:
    out_json["polyfillAliases"][alias].sort()

polyfillAliases = list(out_json["polyfillAliases"].keys())
polyfillAliases.sort()
out_json["polyfillAliases"] = {i: out_json["polyfillAliases"][i] for i in polyfillAliases}


print(json.dumps(out_json))
