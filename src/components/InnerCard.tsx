
import React from 'react';
import {ReactElement} from 'react';
import {
  Accordion,
  AccordionSummary,
  AccordionDetails,
  Box,
  Typography,
} from '@mui/material';


type InnerCardProps = {
  title: string,
  children: React.ReactNode
};

export const InnerCard: React.FC<InnerCardProps> = ({title, children}) => {
  const [expanded, setExpanded] = React.useState<boolean>(false);

  const handleAccordion =
    () => (event: React.SyntheticEvent, newExpanded: boolean) => {
      setExpanded(!expanded);
    };

  return (
    <Accordion square={true} disableGutters={true} sx={{
      border: `1px solid`,
      borderColor: 'divider',
      '&:not(:last-child)': {
        borderBottom: 0,
      },
      '&:before': {
        display: 'none',
      },

    }} expanded={expanded} onChange={handleAccordion()}>

      <AccordionSummary sx={{
        bgcolor: 'info.light',
        flexDirection: 'row-reverse',
        '& .MuiAccordionSummary-expandIconWrapper.Mui-expanded': {
          transform: 'rotate(90deg)',
        },
        '& .MuiAccordionSummary-content': {
          marginLeft: 1,
        },

      }} aria-controls="panel1d-content" id="panel1d-header">
        <Typography>{title}</Typography>
      </AccordionSummary>
      <AccordionDetails sx={{
        padding: 2,
        borderTop: '1px solid rgba(0, 0, 0, .125)',

      }}>
        {children}
      </AccordionDetails>
    </Accordion>
  );
}

