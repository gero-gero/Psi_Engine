# ComfyUI Workflows

Place your ComfyUI **API format** workflow JSON files here.

## IMPORTANT: You must use API format!

The regular "Save" in ComfyUI saves the web/graph format which includes visual
layout data and subgraph references that cannot be sent to the API directly.
You need the **API format** export.

## How to export a workflow in API format

1. Open ComfyUI in your browser
2. Go to **Settings** (gear icon) → Enable **Dev Mode Options**
3. Open your workflow
4. In the menu, click **Save (API Format)** (NOT the regular "Save")
5. Save the `.json` file into this `workflows/` directory
6. The filename (without `.json`) will appear in the game engine's workflow dropdown

## How to tell the difference

- **Web format** (wrong): Has `"nodes"`, `"links"`, `"groups"` arrays at the top level
- **API format** (correct): Has numbered keys like `"1"`, `"2"`, `"3"` each with `"class_type"` and `"inputs"`

## Example API format structure

```json
{
  "6": {
    "class_type": "CLIPTextEncode",
    "inputs": {
      "text": "a beautiful sprite",
      "clip": ["4", 1]
    }
  },
  "7": {
    "class_type": "KSampler",
    "inputs": { ... }
  }
}
```

## Prompt injection

The game engine will automatically replace the text in the first
CLIPTextEncode node (that doesn't look like a negative prompt) with
whatever you type in the Prompt field.
