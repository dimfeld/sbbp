#!/usr/bin/eny python3
import json
import sys
import torch
from transformers import AutoModelForSpeechSeq2Seq, AutoProcessor, pipeline


def run():
    device = (
        "cuda:0"
        if torch.cuda.is_available()
        else "mps"
        if torch.backends.mps.is_available()
        else "cpu"
    )
    torch_dtype = torch.float32 if device == "cpu" else torch.float16

    model_id = "distil-whisper/distil-large-v2"

    model = AutoModelForSpeechSeq2Seq.from_pretrained(
        model_id, torch_dtype=torch_dtype, low_cpu_mem_usage=True, use_safetensors=True
    )
    model.to(device)

    processor = AutoProcessor.from_pretrained(model_id)

    pipe = pipeline(
        "automatic-speech-recognition",
        model=model,
        tokenizer=processor.tokenizer,
        feature_extractor=processor.feature_extractor,
        max_new_tokens=128,
        chunk_length_s=15,
        batch_size=16,
        torch_dtype=torch_dtype,
        device=device,
        return_timestamps=True,
    )

    result = pipe(sys.argv[1])

    print(json.dumps(result["chunks"], indent=2))

