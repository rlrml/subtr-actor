import * as React from "react"
import { cn } from "@/lib/utils"

const Slider = React.forwardRef(({ className, min, max, step, value, onValueChange, onValueCommit, ...props }, ref) => {
    const handleChange = (e) => {
        onValueChange?.([parseFloat(e.target.value)])
    }

    const handleCommit = (e) => {
        onValueCommit?.([parseFloat(e.target.value)])
    }

    return (
        <input
            type="range"
            min={min}
            max={max}
            step={step}
            value={value ? value[0] : 0}
            onChange={handleChange}
            onMouseUp={handleCommit}
            onTouchEnd={handleCommit}
            className={cn(
                "w-full h-2 bg-secondary rounded-lg appearance-none cursor-pointer accent-primary",
                className
            )}
            ref={ref}
            {...props}
        />
    )
})
Slider.displayName = "Slider"

export { Slider }
