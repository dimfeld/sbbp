from skimage.metrics import structural_similarity as ssim
import json
import cv2
import os.path
import glob
import sys


def load_image(path: str):
    img = cv2.imread(path)

    width = img.shape[1]
    if width > 640:
        ratio = 640 / width
        img = cv2.resize(img, (0, 0), fx=ratio, fy=ratio, interpolation=cv2.INTER_AREA)
    return img


def run():
    filenames = glob.glob(sys.argv[1])
    filenames.sort()

    images = [load_image(f) for f in filenames]

    keep = [0]
    for i in range(1, len(images)):
        lastImageIndex = keep[len(keep) - 1]
        lastImage = images[lastImageIndex]
        currentImage = images[i]
        similarity = ssim(lastImage, currentImage, channel_axis=2)

        print(
            f"{os.path.basename(filenames[lastImageIndex])} - {os.path.basename(filenames[i])}: {similarity}",
            file=sys.stderr,
        )

        if similarity < 0.90:
            keep.append(i)

    kept = set(keep)
    removed = [i for i in range(len(images)) if i not in kept]

    output = {
        "removed": list(removed),
        "numImages": len(images),
    }

    print(json.dumps(output))
