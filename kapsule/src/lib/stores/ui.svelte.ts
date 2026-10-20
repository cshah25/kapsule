export const uiState = $state({
  isWizardOpen: false,
  /** Image to preselect the next time the wizard opens (e.g. right after a Library pull). */
  wizardImage: null as string | null,
});

/** Open the Create Vessel wizard, optionally with an image preselected. */
export function openWizard(image?: string) {
  uiState.wizardImage = image ?? null;
  uiState.isWizardOpen = true;
}
