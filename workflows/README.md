# ComfyUI Workflows

Place your ComfyUI workflow JSON files here.

## How to export a workflow from ComfyUI

1. Open your workflow in ComfyUI
2. Click **Save (API Format)** to export the workflow in API format
3. Save the `.json` file into this `workflows/` directory
4. The filename (without `.json`) will appear in the game engine's workflow dropdown

## Example

If you save a file called `my_sprite_workflow.json` here, you can select
`my_sprite_workflow` from the workflow dropdown in the game engine.

**Note:** The workflow must be in ComfyUI's **API format** (not the web/graph format).
You can get this by using "Save (API Format)" in ComfyUI, or by enabling the
"Dev mode Options" in ComfyUI settings which adds the "Save (API Format)" button.
