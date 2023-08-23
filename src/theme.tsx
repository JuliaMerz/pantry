import {createTheme} from '@mui/material/styles';
import {ArrowForwardIosSharp} from '@mui/icons-material';

import {Box} from '@mui/material';
import {styled} from '@mui/material/styles';



import {createContext} from "react";

import type {Theme, ThemeOptions} from '@mui/material/styles';


export const darkTheme: ThemeOptions = {
  palette: {
    mode: 'dark',
    primary: {
      main: '#43a047',
    },
    secondary: {
      main: '#f50057',
    },
  },
};

export const lightTheme: ThemeOptions = {
  palette: {
    mode: 'light',
    primary: {
      main: '#43a047',
    },
    secondary: {
      main: '#f50057',
    },
  },
};

// This is combined with light/dark and then expanded by createTheme
export const universal: ThemeOptions = {
  components: {
    // Name of the component
    MuiButtonBase: {
      defaultProps: {
        // The props to change the default for.
        disableRipple: true, // No more ripple, on the whole application 💣!

      },
    },
    MuiCard: {
      styleOverrides: {
        root: ({ownerState, theme}) => {
          return theme.unstable_sx({
            marginY: 1,
          })
        },
      },
    },
    MuiButton: {
      styleOverrides: {
        root: ({ownerState, theme}) => {
          if (ownerState.variant == "contained" || ownerState.variant == "outlined") {
            return theme.unstable_sx({
              marginRight: 1,
              marginTop: 1,
            })
          }
        },
      },
    },
    MuiTable: {
      defaultProps: {
        size: 'small',
      },
    },
    MuiTableCell: {
      styleOverrides: {
        root: {
          padding: '2px',
        },
      },
    },
    MuiTableContainer: {
      styleOverrides: {
        root: {
          margin: '5px',
          border: '1px solid black',
        },
      },
    },
  },
};

// This is added after createTheme for overrides
export const postUniversal: (theme: Theme) => Theme = (theme) => {
  return createTheme(theme, {
    components: {
      MuiTab: {
        styleOverrides: {
          root: {
            "&.Mui-selected": {
              color: theme.palette.secondary.main,
              "&.Mui-focusVisible": {
              }
            },
          },

        }
      },
      MuiTextField: {
        defaultProps: {
          margin: "dense"
        }
      },
    },

  })
}

interface ColorContextSchema {
  toggleColorMode: () => void;
  color: string;
}

export const ColorContext = createContext<ColorContextSchema>(
  {} as ColorContextSchema
);

export const splitStyle = {
  display: 'flex',
  justifyContent: 'space-between',
};

const flexRowStyle = {
  display: 'flex',
  flexDirection: 'row',
  gap: 2,
};

interface ModalBoxProps {
  children: React.ReactNode;
}

const ModalBoxWrapper = styled(Box)(({theme}) => ({
  position: 'absolute',
  top: '50%',
  left: '50%',
  transform: 'translate(-50%, -50%)',
  width: '80vw',
  bgcolor: theme.palette.background.paper,
  border: `2px solid ${theme.palette.divider}`,
  boxShadow: theme.shadows[24],
  maxHeight: '90vh',
  pt: 2,
  px: 4,
  pb: 3,
  overflow: 'scroll',
  margin: 2,
}));

export function ModalBox (props: {children: any}) {
  return(<ModalBoxWrapper> {props.children}</ModalBoxWrapper >);
};


