import {forwardRef, useRef, useState, useMemo, useEffect, useCallback} from "react";
import {deepmerge} from "@mui/utils";
import {listen} from '@tauri-apps/api/event';
import {produceEmptyRegistryEntry} from './interfaces';
import {Buffer} from 'buffer';
import logoImage from './assets/tauri-icon-bw-128.png';
import {addRegistryEntry, downloadLLM} from './registryHelpers';
import reactLogo from "./assets/react.svg";
import {invoke} from "@tauri-apps/api/tauri";
import {Link, NavLink, NavLinkProps} from 'react-router-dom';
import {lightTheme, darkTheme, universal, postUniversal, ColorContext} from './theme';
import {ErrorContext} from './context';
import PopupState, {bindTrigger, bindMenu} from 'material-ui-popup-state';
import {DeepLinkEvent, toLLMRegistryEntryExternal} from "./interfaces";
import {Notifications} from './Notifications';




import {BrowserRouter as Router, Route, Routes} from "react-router-dom";
import Home from "./pages/Home";
import History from "./pages/History";
import AvailableLLMs from "./pages/AvailableLLMs";
import DownloadLLMs from "./pages/DownloadableLLMs";
import Requests from "./pages/Requests";
import Settings from "./pages/Settings";
import {ModalBox} from './theme';
import LLMInfo from "./components/LLMInfo";

import {
  AppBar,
  Box,
  Button,
  Card,
  CardContent,
  CssBaseline,
  createTheme,
  ThemeProvider,
  InputLabel,
  PaletteMode,
  Toolbar,
  Modal,
  Typography,
  useMediaQuery,
  useTheme,
  Tab,
  Tabs,
  Select,
  MenuItem,
  ListItemButton,
  ListItemText,
  LinearProgress,
} from '@mui/material';

function jsonToBase64(object: any) {
  const json = JSON.stringify(object);
  return Buffer.from(json).toString("base64");
}

function base64ToJson(base64String: string) {
  const json = Buffer.from(base64String, "base64").toString();
  return JSON.parse(json);
}


function LinkTab(props: any) {
  return <Tab component={NavLink} {...props} />;
}



function App() {
  const [mode, setMode] = useState<PaletteMode>("light");
  const [locationText, setLocationText] = useState('Home');
  const [location, setLocation] = useState('home');
  const [latestEvent, setLatestEvent] = useState('test');
  const [downloadModalOpen, setDownloadModalOpen] = useState(false);
  const [lastError, setLastError] = useState('');
  const [downloadRegistryEntry, setDownloadRegistryEntry] = useState(produceEmptyRegistryEntry());
  const [sendErrorFunction, setSendErrorFunction] = useState<(error: string) => void>();



  // stream-id: string, notification

  function colorCalc() {
    return {
      toggleColorMode: () => {
        setMode((prevMode: PaletteMode) =>
          prevMode === "light" ? "dark" : "light"
        );
      },
      color: mode,
    };
  }

  const colorMode = useMemo(
    colorCalc,
    [mode]
  );

  const errorHandler = useMemo(
    () => ({
      sendError: sendErrorFunction ? sendErrorFunction : () => {},
      lastError: lastError
    }), [lastError]);


  function mergeThemes() {
    return postUniversal(createTheme(deepmerge((mode === "light" ? lightTheme : darkTheme), universal)))
  }
  const theme = useMemo(
    mergeThemes,
    [mode]
  );

  const LinkRef = useCallback(forwardRef<HTMLAnchorElement, NavLinkProps>((props, ref) => <NavLink ref={ref} {...props} />), []);

  const MenuItemLink = useCallback((props: {value: string, value2: string, to: string, primary: string}) => {
    const {value2, to, primary} = props;
    return (
      <MenuItem value={value2}>
        <ListItemButton component={LinkRef} to={to}
          onClick={(e) => handleSelectChange(e, value2)}>
          <ListItemText primary={primary} />
        </ListItemButton>
      </MenuItem>
    );
  }, []);

  const handleChange = useCallback((event: any, newValue: string) => {
    setLocation(newValue);
  }, []);

  const handleSelectChange = useCallback((event: any, newValue: string) => {
    setLocation(newValue);
    setLocationText(event.target.outerText);
  }, []);

  const handleBookmark = async () => {
    await addRegistryEntry(downloadRegistryEntry, 'shared');
    setDownloadModalOpen(false);

  }

  const handleDownload = async () => {
    setDownloadModalOpen(false);
    try {
      await addRegistryEntry(downloadRegistryEntry, 'shared');
      await downloadLLM(downloadRegistryEntry, 'shared');
    } catch (error: any) {
      errorHandler.sendError(error.toString());

    }
  }

  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));

  const listenForDeepLink = () => {
    const unlisten_promise = listen<any>("deep-link-request", (raw_event) => {
      setLatestEvent(JSON.stringify(raw_event));

      let event: DeepLinkEvent = raw_event.payload as DeepLinkEvent;

      if (event.payload.type == "DownloadEvent") {



        let registryEntry = toLLMRegistryEntryExternal(base64ToJson(event.payload.base64));


        setDownloadRegistryEntry((current) => {
          return {...downloadRegistryEntry, ...registryEntry}
        })
        setDownloadModalOpen(true);
      }



    });
    return () => {
      // https://github.com/tauri-apps/tauri/discussions/5194#discussioncomment-3651818
      unlisten_promise.then(f => f());
    };
  };
  useEffect(listenForDeepLink);


  return (
    <ColorContext.Provider value={colorMode}>
      <ErrorContext.Provider value={errorHandler}>
        <ThemeProvider theme={theme}>
          <CssBaseline enableColorScheme />


          <Router>
            <AppBar position="sticky">
              <Toolbar>
                <Box
                  component="img"
                  sx={{
                    height: '48px',
                    // width: 350,
                    // maxHeight: { xs: 233, md: 167 },
                    // maxWidth: { xs: 350, md: 250 },
                  }}
                  alt="The house from the offer."
                  src={logoImage}
                />
                <Typography variant="h6" component="div" sx={{flexGrow: 1}}>
                  Pantry
                </Typography>
                {isMobile ? (
                  <>

                    <InputLabel>{locationText}</InputLabel>
                    <Select value={location} >
                      <MenuItemLink value="home" value2="home" to="/" primary="Home" />
                      <MenuItemLink value="available-llms" value2="available-llms" to="/available-llms" primary="Available LLMs" />
                      <MenuItemLink value="download-llms" value2="download-llms" to="/download-llms" primary="Download LLMs" />
                      <MenuItemLink value="requests" value2="requests" to="/requests" primary="Requests" />
                      <MenuItemLink value="settings" value2="settings" to="/settings" primary="Settings" />
                    </Select>
                  </>

                ) : (
                  <Tabs value={location} onChange={handleChange}>
                    <LinkTab label="Home" to="/" value="home" />
                    <LinkTab label="Available LLMs" to="/available-llms" value="available-llms" />
                    <LinkTab label="Download LLMs" to="/download-llms" value="download-llms" />
                    <LinkTab label="Requests" to="/requests" value="requests" />
                    <LinkTab label="Settings" to="/settings" value="settings" />
                  </Tabs>
                )}
              </Toolbar>
              <Notifications onRegisterSendError={setSendErrorFunction} />
            </AppBar>

            <Box sx={{
              p: 3, // padding
              mx: 'auto', // center the Box horizontally
              width: '100%', // Full width
              maxWidth: 'lg', // constrain maximum width to 'lg' breakpoint value
              display: 'flex', // make it a flex container
              flexDirection: 'column', // arrange children vertically
            }}>
              <Routes>
                <Route path="/" element={<Home />} />
                <Route path="/available-llms" element={<AvailableLLMs />} />
                <Route path="/download-llms" element={<DownloadLLMs />} />
                <Route path="/requests" element={<Requests />} />
                <Route path="/settings" element={<Settings />} />
              </Routes>
            </Box>
            <Modal open={downloadModalOpen} onClose={() => setDownloadModalOpen(false)}>
              <ModalBox>

                <Card className="available-llm">
                  <CardContent>
                    <LLMInfo llm={downloadRegistryEntry} rightButton={null} />
                    <Typography variant="body1"><b>Requirements:</b> {downloadRegistryEntry.requirements}</Typography>
                    <Typography variant="body1"><b>User Parameters:</b> {downloadRegistryEntry.userParameters.join(", ")}</Typography>
                    <Typography variant="body1"><b>Capabilities:</b> {JSON.stringify(downloadRegistryEntry.capabilities)}</Typography>

                    <Button onClick={handleBookmark} variant="outlined">Bookmark</Button><Button onClick={handleDownload} variant="contained">Download</Button>
                  </CardContent>
                </Card>
              </ModalBox>
            </Modal>
          </Router>


        </ThemeProvider>
      </ErrorContext.Provider >
    </ColorContext.Provider >

  );
}
App.whyDidYouRender = true;


export default App;


///* <Route path="/history/:id" element={<History />} /> */
//
