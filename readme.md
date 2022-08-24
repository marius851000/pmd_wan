[![forthebadge](https://forthebadge.com/images/badges/uses-git.svg)](https://forthebadge.com)

Licensed under the creative–common 0 license (TODO: just put a LICENSE file)

# pmd_wan
Note that the information written bellow is outdated, and new function has been added. None has been removed, thought. (Still have to update it).

read wan sprites, used in pokemon mystery dungeon: Explorers and Rescue Team (only tested with Explorers of Sky)

reading : production ready. Should never panic, and has been fuzzed

- m_attack.bin can sometimes errors

status of writing sprites : ugly, may work but may panic or produce incorrect result
- monster.bin and m_ground.bin should be both readable and writable
- probably all ground sprite should be too

creating an all new static sprite : Should work with no issue.

## Executing benchs
The benchs use real image not under the license of this repo, that you need to provide yourself.
  * parse use the bulbasaur.wan in the m_ground.bin file. Can be exported with SkyTemple or another .bin EOS extractor.
  * find_fragment use the White Kyurem sprite by FunnyKecleonMeme. It can be downloaded here : (TODO: actually put the download link)