import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpClientModule } from '@angular/common/http';
import { NasService } from './services/nas.service';
import { AdminService } from './services/admin.service';

@NgModule({
  declarations: [],
  imports: [
    CommonModule,
    HttpClientModule
  ],
  providers: [
    NasService,
    AdminService
  ],
  exports: [
    CommonModule
  ]
})
export class SharedModule { }
