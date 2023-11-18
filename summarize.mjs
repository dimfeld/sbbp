export async function summarize(title, transcript) {
  const allText = transcript.map((t) => t.text).join('\n');

  const result = $`promptbox run summarize --title ${title}`;
  result.stdin.write(allText);
  result.stdin.end();

  const output = await result;
  return output.stdout;
}
