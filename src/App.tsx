import { forwardRef, useState, useMemo } from "react";
import { deepmerge } from "@mui/utils";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import { Link, NavLink, NavLinkProps } from 'react-router-dom';
import { lightTheme, darkTheme, universal, postUniversal, ColorContext } from './theme';
import PopupState, { bindTrigger, bindMenu } from 'material-ui-popup-state';



import { BrowserRouter as Router, Route, Routes } from "react-router-dom";
import Home from "./pages/Home";
import History from "./pages/History";
import AvailableLLMs from "./pages/AvailableLLMs";
import DownloadLLMs from "./pages/DownloadableLLMs";
import Requests from "./pages/Requests";
import Settings from "./pages/Settings";

import {
  AppBar,
  Box,
  CssBaseline,
  createTheme,
  ThemeProvider,
  InputLabel,
  PaletteMode,
  Toolbar,
  Typography,
  useMediaQuery,
  useTheme,
  Tab,
  Tabs,
  Select,
  MenuItem,
  ListItemButton,
  ListItemText,
} from '@mui/material';

function LinkTab(props: any) {
  return <Tab component={NavLink} {...props} />;
}

const LinkRef = forwardRef<HTMLAnchorElement, NavLinkProps>((props, ref) => <NavLink ref={ref} {...props} />);

function MenuItemLink(props: any) {
  const { value, to, primary } = props;
  return (
    <MenuItem value={value}>
      <ListItemButton component={LinkRef} to={to}>
        <ListItemText primary={primary} />
      </ListItemButton>
    </MenuItem>
  );
}


function App() {
  const [mode, setMode] = useState<PaletteMode>("light");
  const [value, setValue] = useState('home');


  const colorMode = useMemo(
    () => ({
      toggleColorMode: () => {
        setMode((prevMode: PaletteMode) =>
          prevMode === "light" ? "dark" : "light"
        );
      },
    }),
    []
  );

  const theme = useMemo(
    () => postUniversal(createTheme(deepmerge((mode === "light" ? lightTheme : darkTheme), universal))),
    [mode]
  );


  const handleChange = (event: any, newValue: string) => {
    setValue(newValue);
  };

  const handleSelectChange = (event: any) => {
    setValue(event.target.value);
  };



  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));



  return (
    <ColorContext.Provider value={colorMode}>
      <ThemeProvider theme={theme}>
        <CssBaseline enableColorScheme />


              <Router>
        <AppBar position="static">
          <Toolbar>
            <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
              Logo
            </Typography>
            {isMobile ? (
              <>
              <InputLabel>{value}</InputLabel>
              <Select value={value} onChange={handleSelectChange}>
                <MenuItemLink value="home" to="/" primary="Home" />
                <MenuItemLink value="available-llms" to="/available-llms" primary="Available LLMs" />
                <MenuItemLink value="download-llms" to="/download-llms" primary="Download LLMs" />
                <MenuItemLink value="requests" to="/requests" primary="Requests" />
                <MenuItemLink value="settings" to="/settings" primary="Settings" />
              </Select>
              </>

            ) : (
              <Tabs value={value} onChange={handleChange}>
                <LinkTab label="Home" to="/" value="home" />
                <LinkTab label="Available LLMs" to="/available-llms" value="available-llms" />
                <LinkTab label="Download LLMs" to="/download-llms" value="download-llms" />
                <LinkTab label="Requests" to="/requests" value="requests" />
                <LinkTab label="Settings" to="/settings" value="settings" />
              </Tabs>
            )}
          </Toolbar>
        </AppBar>
      <Box sx={{
          p: 3, // padding
          mx: 'auto', // center the Box horizontally
          width: '100%', // Full width
          maxWidth: 'lg', // constrain maximum width to 'lg' breakpoint value
          bgcolor: 'background.default', // use default background color
          display: 'flex', // make it a flex container
          flexDirection: 'column', // arrange children vertically
        }}>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/history/:id" element={<History />} />
          <Route path="/available-llms" element={<AvailableLLMs />} />
          <Route path="/download-llms" element={<DownloadLLMs />} />
          <Route path="/requests" element={<Requests />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </Box>
      </Router>


      </ThemeProvider>
    </ColorContext.Provider>

  );
}

export default App;


