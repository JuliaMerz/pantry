import LLMInfo from '../components/LLMInfo';
import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { ModalBox } from '../theme';
import {
  Switch,
  Modal,
  Button,
  Link,
  Typography,
  Card,
  CardContent,
} from '@mui/material/';
import {LLMAvailable} from '../interfaces';

type LLMAvailableInfoProps = {
  llm: LLMAvailable
}

const LLMAvailableInfo: React.FC<LLMAvailableInfoProps> = ({
  llm
}) => {

  // Use this for enabling the LLM
  const [checked, setChecked] = React.useState(false);
  const [openModal, setOpenModal] = useState(false);

  const handleOpenModal = () => {
    setOpenModal(true);
  };

  const handleCloseModal = () => {
    setOpenModal(false);
  };

  const handleConfirmDelete = async () => {
    setOpenModal(false);
    const result = await invoke('delete_llm', { uuid: llm.uuid });
    console.log("deleted", llm.id, result);
  };

  const handleToggle = async () => {
    // call function to disable the LLM
    console.log("Enable the LLM");
    if (!checked) {
      const result = await invoke('load_llm', {uuid: llm.uuid});
      console.log(result);
    } else {
      const result = await invoke('unload_llm', {uuid: llm.uuid});
      console.log(result);
    }
    setChecked(!checked);
  };
  return (
    <Card variant="outlined" sx={{ boxShadow: 1, p: 2, paddingTop: 0, marginBottom:2 }}>
      <CardContent>
      <LLMInfo key={llm.id} llm={llm} rightButton={<Switch checked={checked} onClick={handleToggle}/> }/>

      <Link href={"/history/"+llm.id}>Last Called: {llm.lastCalled ? llm.lastCalled.toString() : "Never"}</Link>
    <Typography variant="body2"><small>Downloaded: {llm.downloaded}</small></Typography>
          <Button variant="contained" onClick={handleOpenModal} color="error">Delete</Button>

          <Modal
        open={openModal}
        onClose={handleCloseModal}
        aria-labelledby="delete-confirmation-modal"
        aria-describedby="delete-confirmation-modal-description"
      >
            <ModalBox>
          <Card className="delete-llm">
            <CardContent>
          <Typography variant="h6" id="delete-confirmation-modal">Confirm Delete</Typography>
          <Typography variant="body1" id="delete-confirmation-modal-description">Are you sure you want to delete this item?</Typography>
          <Button variant="contained" onClick={handleConfirmDelete}>Yes</Button>
          <Button variant="outlined" onClick={handleCloseModal}>No</Button>
            </CardContent>
          </Card>
        </ModalBox>
      </Modal>

    </CardContent>
    </Card>
  )
}

export default LLMAvailableInfo;
