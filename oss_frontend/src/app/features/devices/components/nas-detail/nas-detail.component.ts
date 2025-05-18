import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { NasService } from '../../../../shared/services/nas.service';
import { Nas } from '../../../../shared/models/nas.model';
import {TranslatePipe} from '@ngx-translate/core';
import {DatePipe} from '@angular/common';

@Component({
  selector: 'app-nas-detail',
  templateUrl: './nas-detail.component.html',
  imports: [
    TranslatePipe,
    DatePipe,
  ],
  styleUrls: ['./nas-detail.component.scss'],
})
export class NasDetailComponent implements OnInit {
  nas: Nas | null = null;
  loading = false;
  error = '';

  constructor(
    private nasService: NasService,
    private route: ActivatedRoute,
    private router: Router
  ) { }

  ngOnInit(): void {
    this.loadNasDetails();
  }

  loadNasDetails(): void {
    this.loading = true;
    this.error = '';

    const id = this.route.snapshot.paramMap.get('id');
    if (!id) {
      this.error = 'Invalid NAS ID';
      this.loading = false;
      return;
    }

    this.nasService.getNasById(+id)
      .subscribe({
        next: (nas) => {
          this.nas = nas;
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load NAS details. Please try again later.';
          console.error('Error loading NAS details:', err);
          this.loading = false;
        }
      });
  }

  editNas(): void {
    if (this.nas) {
      this.router.navigate(['/devices/nas', this.nas.id, 'edit']);
    }
  }

  deleteNas(): void {
    if (!this.nas) return;

    if (confirm('Are you sure you want to delete this NAS device?')) {
      this.nasService.deleteNas(this.nas.id)
        .subscribe({
          next: () => {
            this.router.navigate(['/devices/nas']);
          },
          error: (err) => {
            this.error = 'Failed to delete NAS device. Please try again later.';
            console.error('Error deleting NAS device:', err);
          }
        });
    }
  }

  goBack(): void {
    this.router.navigate(['/devices/nas']);
  }
}
