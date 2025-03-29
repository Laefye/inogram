--
-- PostgreSQL database dump
--

-- Dumped from database version 17.4
-- Dumped by pg_dump version 17.4

-- Started on 2025-03-29 12:38:01

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- TOC entry 217 (class 1259 OID 25049)
-- Name: items; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.items (
    id bigint NOT NULL,
    user_id bigint,
    message_id bigint
);


--
-- TOC entry 218 (class 1259 OID 25052)
-- Name: items_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

ALTER TABLE public.items ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.items_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- TOC entry 224 (class 1259 OID 25094)
-- Name: known; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.known (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    item_id bigint NOT NULL
);


--
-- TOC entry 223 (class 1259 OID 25093)
-- Name: known_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

ALTER TABLE public.known ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.known_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- TOC entry 219 (class 1259 OID 25053)
-- Name: messages; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.messages (
    id bigint NOT NULL,
    from_id bigint NOT NULL,
    chat_id bigint NOT NULL,
    text text,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- TOC entry 220 (class 1259 OID 25059)
-- Name: messages_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

ALTER TABLE public.messages ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.messages_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- TOC entry 221 (class 1259 OID 25060)
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users (
    id bigint NOT NULL,
    email text NOT NULL,
    username text,
    first_name text NOT NULL,
    last_name text,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


--
-- TOC entry 222 (class 1259 OID 25066)
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

ALTER TABLE public.users ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.users_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- TOC entry 4760 (class 2606 OID 25068)
-- Name: items items_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.items
    ADD CONSTRAINT items_pkey PRIMARY KEY (id);


--
-- TOC entry 4766 (class 2606 OID 25098)
-- Name: known known_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.known
    ADD CONSTRAINT known_pkey PRIMARY KEY (id);


--
-- TOC entry 4762 (class 2606 OID 25070)
-- Name: messages messages_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.messages
    ADD CONSTRAINT messages_pkey PRIMARY KEY (id);


--
-- TOC entry 4764 (class 2606 OID 25072)
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- TOC entry 4769 (class 2606 OID 25073)
-- Name: messages chat_id_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.messages
    ADD CONSTRAINT chat_id_fk FOREIGN KEY (chat_id) REFERENCES public.items(id) NOT VALID;


--
-- TOC entry 4770 (class 2606 OID 25078)
-- Name: messages from_id_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.messages
    ADD CONSTRAINT from_id_fk FOREIGN KEY (from_id) REFERENCES public.items(id);


--
-- TOC entry 4771 (class 2606 OID 25104)
-- Name: known item_id_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.known
    ADD CONSTRAINT item_id_fk FOREIGN KEY (item_id) REFERENCES public.items(id);


--
-- TOC entry 4767 (class 2606 OID 25083)
-- Name: items message_id_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.items
    ADD CONSTRAINT message_id_fk FOREIGN KEY (message_id) REFERENCES public.messages(id) NOT VALID;


--
-- TOC entry 4768 (class 2606 OID 25088)
-- Name: items user_id_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.items
    ADD CONSTRAINT user_id_fk FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- TOC entry 4772 (class 2606 OID 25099)
-- Name: known user_id_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.known
    ADD CONSTRAINT user_id_fk FOREIGN KEY (user_id) REFERENCES public.items(id);


-- Completed on 2025-03-29 12:38:01

--
-- PostgreSQL database dump complete
--

