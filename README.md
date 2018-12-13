# PhotoArchiver
- Sorts some media files (JPG, TIFF, PNG, MOV, and MP4) using Google Photos folder structure (YYYY/MMM/dd).
- Generates a log file showing which file were moved and where.
- The app will look for the EXIF tag 36867 (JPG and TIFF only), if not present, the app will use the "date modified" metadata.
