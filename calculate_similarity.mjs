const pythonDir = path.join(__dirname, 'python');

async function calculateSimilarity(image1, image2) {
  image1 = path.resolve(image1);
  image2 = path.resolve(image2);
  return within(async () => {
    cd(pythonDir);
    const sim = await $`rye run compare-images ${image1} ${image2}`;
    return parseFloat(sim);
  });
}

export async function removeSimilarImages(images) {
  let keep = [0];
  for(let i = 1; i < images.length; i++) {
    const lastImageIndex = keep[keep.length - 1];
    const lastImage = images[lastImageIndex];
    const currentImage = images[i];
    const similarity = await calculateSimilarity(lastImage, currentImage);

    if(similarity < 0.90) {
      // Don't actually delete the image for now, to make it easier to tweak the removal algorithm without redownloading.
      keep.push(i);
    }
  }

  const kept = new Set(keep);
  const removed = Array.from({ length: images.length }, (_, i) => i).filter((i) => !kept.has(i));

  return removed;
}
