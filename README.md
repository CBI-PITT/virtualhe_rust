# virtualhe_rust

## This tool creates virtual H&E images from fluorescent microscopy data

The implementation is is written in rust, is NOT GPU accelerated, and is based on the following paper:

Giacomelli MG, Husvogt L, Vardeh H,  Faulkner-Jones BE, Hornegger J, Connolly JL, Fujimoto JG. Virtual  Hematoxylin and Eosin Transillumination Microscopy Using  Epi-Fluorescence Imaging. PLoS One. 2016 Aug 8;11(8):e0159337. doi:  10.1371/journal.pone.0159337. PMID: 27500636; PMCID: PMC4976978.

###### Description:

- Precompiled binaries are available for:

  - Windows: bin/win/virtualhe.exe
  - Linux: bin/linux/virtualhe

  A Python implementation can be found [here](https://github.com/CBI-PITT/virtualhe_py).

This tool takes a 16bit or 8bit greyscale image of nuclei (vHematoxylin) and a 16bit or 8bit greyscale background image (vEosin) like autofluorescence or a fluorescent counterstain like eosin. The output in a 8bit RGB image saved to disk.

###### Usage example:

```bash
bin/win/virtualhe.exe --help

--- BEGIN OUTPUT ---

Make a Virtual H&E Image from Fluorescent Microscopy Images

Usage: virtualhe.exe [OPTIONS] <NUCLEUS> <EOSIN> <OUTPUT>

Arguments:
  <NUCLEUS>  Path to the nucleus (hematoxylin) channel image (e.g., nucleus.tif)
  <EOSIN>    Path to the eosin channel image (e.g., autof.tif)
  <OUTPUT>   Path to save the output RGB image (e.g., output.tiff)

Options:
  -k <K>      K arbitrary factor to adjust color profile of H&E [default: 2.5]
  -h, --help  Print help

--- END OUTPUT ---

```

- Now we will take 2 images:
  - Nuclear Image (i.e. DAPI, sytox, topro3): c:\data\images\nucleus_image.tif
  - Background Image (i.e. autofluorescence, eosin): c:\data\images\vhe_output.tif
- We want the output image to be located here:
  - c:\data\images\vhe_output.tif

```bash
# Run this command
python c:\virtualhe\bin\win\virtualhe.exe c:\data\images\nucleus_image.tif c:\data\images\background_image.tif c:\data\images\vhe_output.tif

--- BEGIN OUTPUT ---
Reading c:\data\images\nucleus_image.tif
Reading c:\data\images\background_image.tif
Scaling channels
Calculating and Saving vH&E
Virtual H&E image saved to: c:\data\images\vhe_output.tif
--- END OUTPUT ---
```

###### NOTE:

- Memory: The amount of memory required to process is approximately 10x the size on any 1 uncompressed 16bit input image and 20x the size on any 1 uncompressed 8bit image. 
  - For the example above, if the 16bit image "nucleus_image.tif" is 1 gigabyte, it will require ~10 gigabytes of RAM to process (1gigabyte * 10). 
  - If the image was 8bit, it would require ~20 gigabytes of RAM to process (1gigabyte * 20). 
