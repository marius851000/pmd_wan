[![forthebadge](https://forthebadge.com/images/badges/uses-git.svg)](https://forthebadge.com)

Licensed under the creative–common 0 license (TODO: just put a LICENSE file)

# pmd_wan
Note that the information written bellow is outdated, and new function has been added. None has been removed, thought. (Still have to update it).

read wan sprites, used in pokemon mystery dungeon: Explorers and Rescue Team (only tested with Explorers of Sky)

reading : production ready. Should never panic, and has been fuzzed

- m_attack.bin can sometimes errors

writing : does not provide a nice API for high level stuff for now, but should work correctly, and produce readable images by the game

# Shiren
I’m currently trying to read images from the Shiren The Wanderer on DS. They are similar in some point, and dissimilar in others.

For now, I focus on being able to read them.

It’s behind a feature flag. It is mostly experimental code for now.

## Executing benchs
The benchs use real image not under the license of this repo, that you need to provide yourself.
  * parse use the bulbasaur.wan in the m_ground.bin file. Can be exported with SkyTemple or another .bin EOS extractor.
  * find_fragment use the White Kyurem sprite by FunnyKecleonMeme. It can be downloaded here : (TODO: actually put the download link)