import json
import os
import sys


def main() -> None:
    """
    Main entry point for the plugin.

    Environment Variables:
        ANTHILL_PLUGIN_PARAMS: JSON string containing user parameters
        ANTHILL_PHASE: Either "prepare" (preview) or "apply" (execute)
        ANTHILL_PREVIEW_PLAN: JSON string with preview data (apply phase only)
    """
    # Get environment variables
    raw_params = os.getenv("ANTHILL_PLUGIN_PARAMS")
    phase = os.getenv("ANTHILL_PHASE", "apply")
    preview_plan = os.getenv("ANTHILL_PREVIEW_PLAN")

    # Parse parameters
    params = {}
    if raw_params:
        try:
            params = json.loads(raw_params)
        except json.JSONDecodeError:
            params = {"_raw": raw_params}

    # === PREPARE PHASE (Preview) ===
    if phase == "prepare":
        # Generate and output a preview plan
        plan = {
            "phase": "prepare",
            "message": "Preview of what will happen",
            "targets": [],  # List items that will be affected
            # Add any other preview information here
        }
        print(json.dumps(plan, ensure_ascii=False))
        return

    # === APPLY PHASE (Execute) ===
    if phase == "apply":
        # Execute the actual operation
        print("Executing plugin...")

        # Access user parameters
        example_param = params.get("example_param", "default_value")
        print(f"example_param: {example_param}")

        # Access preview plan if available
        if preview_plan:
            try:
                plan = json.loads(preview_plan)
                print(f"Applying preview plan: {plan}")
            except json.JSONDecodeError:
                print(f"Preview plan (raw): {preview_plan}")

        # Your plugin logic here
        # ...

        print("Plugin execution complete.")
        return

    # Unknown phase
    print(f"Unknown ANTHILL_PHASE: {phase}", file=sys.stderr)
    sys.exit(1)


if __name__ == "__main__":
    main()
