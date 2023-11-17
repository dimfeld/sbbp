from skimage.metrics import structural_similarity as ssim
import cv2
import sys


def run():
    img1 = cv2.imread(sys.argv[1])
    img2 = cv2.imread(sys.argv[2])
    sim = ssim(img1, img2, channel_axis=2)

    print(sim)
