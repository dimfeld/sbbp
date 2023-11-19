const pythonDir = path.join(__dirname, 'python');

export async function removeSimilarImages(imageGlob) {
  imageGlob = path.resolve(imageGlob);
  const result = await within(async () => {
    cd(pythonDir);
    return await $`rye run compare-images ${imageGlob}`;
  });

  return JSON.parse(result.stdout);
}
