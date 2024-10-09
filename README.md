# Obsidian -> mdbook | Parser

This project aims to help me learn rust further and provide a mean to update my website directly from my given obsidian vault, by automatically creating a valid **SUMMARY.md** for mdbook and providing a list which files to copy.

---

## Overview 

This converter helps to **convert** an obsidian vault to a valid **mdbook** representation.

### Configuring:

settings can be set in [src/settings.rs].

Those include:
- Path to **traverse and create** representation from
- Path to copy files to
- Path to save **SUMMARY.md** to --> required for mdbook to create TOC
- Path where **settings** such as _excluded files_ or _headline_prefixes_ are set (and can be read)

### TODO:

- implement setting prefixes for headlines
- (maybe) have context sensitive prefixes for headlines
- improve code quality further 

#### Archive

- ~~adapt paths given to **new** base-directory (after all they should be copied to the corresponding md-book structure src/)~~

- ~~use rsync to copy all collected files accordingly.~~

### MISC

I'm trying x) 

Sorry for the code quality //