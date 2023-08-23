import {forwardRef, useRef, useState, useMemo, useEffect, useCallback} from "react";
import {listen} from '@tauri-apps/api/event';
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


interface OngoingNotification {
  lastId: number,
  progress?: number | string,
  description: string,
  type: "error" | "download" | "inference" | "notification"
  timeout: number,
}

export const Notifications: React.FC<{onRegisterSendError?: (sendErrorFunction: (error: string) => void) => void}> = ({onRegisterSendError}) => {
  const [ongoingNotifications, setOngoingNotifications] = useState<{[key: string]: OngoingNotification}>({});
  const refNotifications = useRef(ongoingNotifications);

  function updateNotifications() {
    setOngoingNotifications((prev) => {
      let newNotifs: {[key: string]: OngoingNotification} = {}
      for (const [streamId, val] of Object.entries(prev)) {
        if (val.timeout > Date.now()) {
          newNotifs[streamId] = val;
        }
      }
      return newNotifs;
    });
  }

  useEffect(() => {
    let i = setInterval(updateNotifications, 1000);
    return () => {
      clearInterval(i);
    };
  }, []);


  const sendError = useCallback((error: string) => {
    let msgId = Math.random()
    setOngoingNotifications((prev) => {
      let new_error: OngoingNotification = {
        lastId: msgId,
        description: error,
        type: "error",
        timeout: Date.now() + 3000
      }
      return {[msgId]: new_error, ...prev}
    });
    setTimeout(() => {
      setOngoingNotifications((prev) => {
        const {[msgId]: _, ...without} = prev;
        return without;
      });
    }, 3000);
  }, [setOngoingNotifications]);

  useEffect(() => {
    if (onRegisterSendError) {
      onRegisterSendError(sendError);
    }
  }, [onRegisterSendError, sendError]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let unlisten2: (() => void) | undefined;
    (async () => {
      unlisten = await listen('downloads', (event: any) => {
        const msgId = Math.random();
        const streamId = event.payload.stream_id;
        if (streamId in refNotifications.current && event.payload.event.type == "DownloadProgress") {
          setOngoingNotifications((prev) => {
            if (prev[streamId]) {
              prev[streamId].lastId = msgId;
              prev[streamId].progress = parseInt(event.payload.event.progress);
              prev[streamId].description = "Downloading LLM ";
              prev[streamId].timeout = Date.now() + 3000;
            } else {
            }
            return {...prev};

          });
        } else {
          setOngoingNotifications((prev) => {
            prev[streamId] = {
              lastId: msgId,
              progress: parseInt(event.payload.event.progress),
              description: "Downloading LLM ",
              type: "download",
              timeout: Date.now() + 3000
            };
            return {...prev}
          });
        }

      });
    })();

    (async () => {
      unlisten = await listen('notification', (event: any) => {
        const msgId = Math.random();
        const streamId = event.payload.stream_id;
        if (streamId in refNotifications.current) {
          setOngoingNotifications((prev) => {
            if (prev[streamId]) {
              prev[streamId].lastId = msgId;
              prev[streamId].description = event.payload.event.message;
              prev[streamId].timeout = Date.now() + 6000;
            } else {
            }
            return {...prev};

          });
        } else {
          setOngoingNotifications((prev) => {
            prev[streamId] = {
              lastId: msgId,
              description: event.payload.event.message,
              type: "notification",
              timeout: Date.now() + 6000
            };
            return {...prev}
          });
        }

      });
    })();

    return () => {unlisten && unlisten();}
  });


  return (
    <Box>
      {Object.keys(ongoingNotifications).map((streamId, index) => (
        <Box sx={{
          display: 'flex',
          flexDirection: 'row',
          alignItems: 'center',
          paddingX: 1,
          background: "background.main",
          paddingY: 0.5,
        }} key={streamId}>
          <Typography color={(ongoingNotifications[streamId].type == "error" ? "error.main" : "text.primary")} sx={{
            marginRight: 2,
          }}>{ongoingNotifications[streamId].description}</Typography>
          {ongoingNotifications[streamId].progress ? <LinearProgress sx={{
            flexGrow: 10,


          }}
            color="info"
            variant="determinate" value={parseInt(ongoingNotifications[streamId].progress as string)} /> : <LinearProgress color="info" sx={{

            }} variant="indeterminate" />}

        </Box>
      ))}

    </Box>
  );
  //
}

