declare module "@mui/material/styles" {
  interface AccordionDetailsVariants {
    innerCard: React.CSSProperties;
  }

  // allow configuration using `createTheme`
  interface AccordionDetailsVariantsOptions {
    innerCard?: React.CSSProperties;
  }
  interface AccordionVariants {
    innerCard: React.CSSProperties;
  }

  // allow configuration using `createTheme`
  interface AccordionVariantsOptions {
    innerCard?: React.CSSProperties;
  }
  interface AccordionSummaryVariants {
    innerCard: React.CSSProperties;
  }

  // allow configuration using `createTheme`
  interface AccordionSummaryVariantsOptions {
    innerCard?: React.CSSProperties;
  }
}

// Update the Typography's variant prop options
declare module "@mui/material/AccordionDetails" {
  interface AccordionDetailsPropsVariantOverrides {
    innerCard: true;
  }
}
declare module "@mui/material/AccordionSummary" {
  interface AccordionSummaryPropsVariantOverrides {
    innerCard: true;
  }
}
declare module "@mui/material/Accordion" {
  interface AccordionPropsVariantOverrides {
    innerCard: true;
  }
}
