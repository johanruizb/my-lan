// Spacing scale unificado (elimina drift gap-2/gap-4/gap-6 en section containers).
// gap-2 = button groups (intencional, items-center apretados).
// gap-4 = section containers (default).
// gap-6 = solo CardContent mayor.
export const SPACING = {
    gapXs: "gap-2",
    gapSm: "gap-3",
    gapMd: "gap-4",
    gapLg: "gap-6",
} as const;

export const SECTION_GAP = SPACING.gapMd;
export const BUTTON_GROUP_GAP = SPACING.gapXs;
