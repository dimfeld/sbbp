#!/usr/bin/env zx

import { summarize } from './summarize.mjs';

const dir = process.argv[3];

const config = JSON.parse(fs.readFileSync(path.join(dir, 'sbbp.json')).toString());
const transcript = JSON.parse(fs.readFileSync(path.join(dir, 'transcript.json')).toString());

if(!fs.existsSync(dir)) {
  throw new Error('Directory does not exist');
}

const summary = await summarize(config.title, transcript);

config.summary = summary;

await fs.writeFile(path.join(dir, 'sbbp.json'), JSON.stringify(config, null, 2));

